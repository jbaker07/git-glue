//! Production-ready mapping module: transform and enrich normalized events.

use crate::adapter::NormalizedEvent;
use crate::fact::{Fact, Severity};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Mapping Configuration
// ============================================================================

/// Configuration for event mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfig {
    /// Severity mapping rules
    pub severity_rules: HashMap<String, Severity>,
    /// Event type normalization rules
    pub type_normalization: HashMap<String, String>,
    /// Primitive extraction rules
    pub primitive_rules: Vec<PrimitiveRule>,
    /// MITRE ATT&CK mapping
    pub mitre_mapping: HashMap<String, Vec<String>>,
}

impl Default for MappingConfig {
    fn default() -> Self {
        let mut severity_rules = HashMap::new();
        severity_rules.insert("critical".to_string(), Severity::Critical);
        severity_rules.insert("high".to_string(), Severity::High);
        severity_rules.insert("medium".to_string(), Severity::Medium);
        severity_rules.insert("low".to_string(), Severity::Low);
        severity_rules.insert("informational".to_string(), Severity::Informational);
        severity_rules.insert("info".to_string(), Severity::Informational);

        Self {
            severity_rules,
            type_normalization: HashMap::new(),
            primitive_rules: default_primitive_rules(),
            mitre_mapping: default_mitre_mapping(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveRule {
    /// Event type pattern (regex or exact)
    pub event_type_pattern: String,
    /// Primitive to assign
    pub primitive: String,
    /// Confidence boost/penalty
    pub confidence_modifier: f32,
}

fn default_primitive_rules() -> Vec<PrimitiveRule> {
    vec![
        PrimitiveRule {
            event_type_pattern: "(?i)process.*creat".to_string(),
            primitive: "process_creation".to_string(),
            confidence_modifier: 0.0,
        },
        PrimitiveRule {
            event_type_pattern: "(?i)network.*connect".to_string(),
            primitive: "network_connection".to_string(),
            confidence_modifier: 0.0,
        },
        PrimitiveRule {
            event_type_pattern: "(?i)file.*write|file.*modif".to_string(),
            primitive: "file_modification".to_string(),
            confidence_modifier: 0.0,
        },
        PrimitiveRule {
            event_type_pattern: "(?i)dns".to_string(),
            primitive: "dns_query".to_string(),
            confidence_modifier: 0.0,
        },
        PrimitiveRule {
            event_type_pattern: "(?i)registry".to_string(),
            primitive: "registry_modification".to_string(),
            confidence_modifier: 0.0,
        },
        PrimitiveRule {
            event_type_pattern: "(?i)logon|login|auth".to_string(),
            primitive: "authentication".to_string(),
            confidence_modifier: 0.0,
        },
    ]
}

fn default_mitre_mapping() -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    map.insert(
        "process_creation".to_string(),
        vec!["T1059".to_string(), "T1106".to_string()],
    );
    map.insert(
        "network_connection".to_string(),
        vec!["T1071".to_string(), "T1095".to_string()],
    );
    map.insert(
        "file_modification".to_string(),
        vec!["T1027".to_string(), "T1070".to_string()],
    );
    map.insert(
        "registry_modification".to_string(),
        vec!["T1547".to_string(), "T1112".to_string()],
    );
    map.insert("dns_query".to_string(), vec!["T1071.004".to_string()]);
    map.insert("authentication".to_string(), vec!["T1078".to_string()]);
    map
}

// ============================================================================
// Event Mapper
// ============================================================================

pub struct EventMapper {
    config: MappingConfig,
}

impl EventMapper {
    pub fn new(config: MappingConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(MappingConfig::default())
    }

    /// Map a normalized event to a fact
    pub fn event_to_fact(&self, event: &NormalizedEvent) -> Fact {
        let primitive = self.determine_primitive(&event.event_type);
        let severity = self.map_severity(event.severity.as_deref());
        let confidence = self.calculate_confidence(event, &primitive);

        let mut fact = Fact::new(
            &primitive,
            &self.generate_description(event),
            &event.source,
            confidence,
        );
        fact.severity = Some(severity);
        fact.timestamp = Some(event.timestamp);
        fact.raw_event_id = Some(event.id.clone());

        // Add metadata
        if let Some(proc) = &event.process {
            if let Some(name) = &proc.name {
                fact.metadata
                    .insert("process_name".to_string(), serde_json::json!(name));
            }
            if let Some(cmd) = &proc.command_line {
                fact.metadata
                    .insert("command_line".to_string(), serde_json::json!(cmd));
            }
        }

        if let Some(net) = &event.network {
            if let Some(ip) = &net.dest_ip {
                fact.metadata
                    .insert("dest_ip".to_string(), serde_json::json!(ip));
            }
        }

        fact
    }

    /// Map multiple events to facts
    pub fn events_to_facts(&self, events: &[NormalizedEvent]) -> Vec<Fact> {
        events.iter().map(|e| self.event_to_fact(e)).collect()
    }

    /// Determine primitive from event type
    fn determine_primitive(&self, event_type: &str) -> String {
        for rule in &self.config.primitive_rules {
            if let Ok(re) = regex::Regex::new(&rule.event_type_pattern) {
                if re.is_match(event_type) {
                    return rule.primitive.clone();
                }
            }
        }

        // Fallback: use normalized event type
        event_type.to_lowercase().replace(' ', "_")
    }

    /// Map severity string to enum
    fn map_severity(&self, severity: Option<&str>) -> Severity {
        severity
            .and_then(|s| self.config.severity_rules.get(&s.to_lowercase()))
            .copied()
            .unwrap_or(Severity::Medium)
    }

    /// Calculate confidence based on event quality
    fn calculate_confidence(&self, event: &NormalizedEvent, primitive: &str) -> f32 {
        let mut confidence = 0.7; // Base confidence

        // Boost for detailed process info
        if let Some(proc) = &event.process {
            if proc.command_line.is_some() {
                confidence += 0.1;
            }
            if proc.path.is_some() {
                confidence += 0.05;
            }
        }

        // Boost for severity info
        if event.severity.is_some() {
            confidence += 0.05;
        }

        // Apply primitive-specific rules
        for rule in &self.config.primitive_rules {
            if rule.primitive == primitive {
                confidence += rule.confidence_modifier;
                break;
            }
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Generate human-readable description
    fn generate_description(&self, event: &NormalizedEvent) -> String {
        let mut parts = Vec::new();

        parts.push(format!("{} event", event.event_type));

        if let Some(proc) = &event.process {
            if let Some(name) = &proc.name {
                parts.push(format!("process: {}", name));
            }
        }

        parts.push(format!("on {}", event.host));

        if let Some(user) = &event.user {
            parts.push(format!("by {}", user));
        }

        parts.join(" | ")
    }

    /// Get MITRE techniques for a primitive
    pub fn get_mitre_techniques(&self, primitive: &str) -> Vec<String> {
        self.config
            .mitre_mapping
            .get(primitive)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for EventMapper {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Map events using default configuration
pub fn map_events(events: &[NormalizedEvent]) -> Vec<Fact> {
    EventMapper::with_defaults().events_to_facts(events)
}

/// Map a single event to internal format (backward compatible)
pub fn map_event_to_internal(event: &serde_json::Value) -> serde_json::Value {
    // Legacy function - just return as-is
    event.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_detection() {
        let mapper = EventMapper::with_defaults();
        assert_eq!(
            mapper.determine_primitive("ProcessCreation"),
            "process_creation"
        );
        assert_eq!(
            mapper.determine_primitive("NetworkConnection"),
            "network_connection"
        );
    }

    #[test]
    fn test_severity_mapping() {
        let mapper = EventMapper::with_defaults();
        assert_eq!(mapper.map_severity(Some("critical")), Severity::Critical);
        assert_eq!(mapper.map_severity(Some("HIGH")), Severity::High);
        assert_eq!(mapper.map_severity(None), Severity::Medium);
    }
}
