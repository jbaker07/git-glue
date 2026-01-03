//! Production-ready coverage-scoped Fact and Incident models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Fact Model
// ============================================================================

/// A single piece of evidence or assertion derived from telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Unique identifier
    pub id: String,
    /// The primitive/category this fact relates to (e.g., "process_creation", "network_connection")
    pub primitive: String,
    /// Human-readable description
    pub description: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Source that provided this fact (e.g., "CrowdStrike", "SentinelOne")
    pub source: String,
    /// Additional sources corroborating this fact
    pub corroborating_sources: Vec<String>,
    /// Evidence types needed for full confirmation
    pub requirements: Vec<String>,
    /// What evidence is missing
    pub missing_inputs: Vec<String>,
    /// How/where this fact was derived
    pub provenance: Option<String>,
    /// When the underlying event occurred
    pub timestamp: Option<DateTime<Utc>>,
    /// When this fact was created/ingested
    pub ingested_at: DateTime<Utc>,
    /// Severity level
    pub severity: Option<Severity>,
    /// Analyst notes
    pub analyst_notes: Option<String>,
    /// Raw data reference
    pub raw_event_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Fact {
    pub fn new(primitive: &str, description: &str, source: &str, confidence: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            primitive: primitive.to_string(),
            description: description.to_string(),
            confidence,
            source: source.to_string(),
            corroborating_sources: Vec::new(),
            requirements: Vec::new(),
            missing_inputs: Vec::new(),
            provenance: None,
            timestamp: None,
            ingested_at: Utc::now(),
            severity: None,
            analyst_notes: None,
            raw_event_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn with_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = Some(ts);
        self
    }

    pub fn add_corroboration(&mut self, source: &str) {
        if !self.corroborating_sources.contains(&source.to_string()) {
            self.corroborating_sources.push(source.to_string());
            // Boost confidence slightly for each corroboration
            self.confidence = (self.confidence + 0.1).min(1.0);
        }
    }
}

// ============================================================================
// Incident Model
// ============================================================================

/// A compiled incident combining multiple facts with analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    /// Unique identifier
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Overall severity
    pub severity: Severity,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Current status
    pub status: IncidentStatus,
    /// Supporting facts
    pub facts: Vec<Fact>,
    /// Sources present in this incident
    pub coverage_scope: Vec<String>,
    /// When the incident started (earliest fact timestamp)
    pub started_at: Option<DateTime<Utc>>,
    /// When the incident ended (latest fact timestamp, if closed)
    pub ended_at: Option<DateTime<Utc>>,
    /// When this incident was created
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// Detected contradictions between sources
    pub contradictions: Vec<String>,
    /// Coverage gaps (missing expected data)
    pub coverage_gaps: Vec<String>,
    /// Explainability: why we believe this is an incident
    pub why_explanation: Option<String>,
    /// Analyst override notes
    pub analyst_override: Option<String>,
    /// MITRE ATT&CK techniques
    pub mitre_techniques: Vec<String>,
    /// Related host names/IPs
    pub hosts: Vec<String>,
    /// Related user accounts
    pub users: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Incident {
    pub fn new(title: &str, description: &str, severity: Severity) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: description.to_string(),
            severity,
            confidence: 0.0,
            status: IncidentStatus::New,
            facts: Vec::new(),
            coverage_scope: Vec::new(),
            started_at: None,
            ended_at: None,
            created_at: now,
            updated_at: now,
            contradictions: Vec::new(),
            coverage_gaps: Vec::new(),
            why_explanation: None,
            analyst_override: None,
            mitre_techniques: Vec::new(),
            hosts: Vec::new(),
            users: Vec::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a fact and update incident metadata
    pub fn add_fact(&mut self, fact: Fact) {
        // Update coverage scope
        if !self.coverage_scope.contains(&fact.source) {
            self.coverage_scope.push(fact.source.clone());
        }
        for source in &fact.corroborating_sources {
            if !self.coverage_scope.contains(source) {
                self.coverage_scope.push(source.clone());
            }
        }

        // Update time range
        if let Some(ts) = fact.timestamp {
            match self.started_at {
                None => self.started_at = Some(ts),
                Some(start) if ts < start => self.started_at = Some(ts),
                _ => {}
            }
        }

        // Recalculate confidence
        self.facts.push(fact);
        self.recalculate_confidence();
        self.updated_at = Utc::now();
    }

    /// Recalculate overall confidence based on facts
    fn recalculate_confidence(&mut self) {
        if self.facts.is_empty() {
            self.confidence = 0.0;
            return;
        }

        // Weighted average with source diversity bonus
        let avg_confidence: f32 =
            self.facts.iter().map(|f| f.confidence).sum::<f32>() / self.facts.len() as f32;
        let source_bonus = (self.coverage_scope.len() as f32 * 0.05).min(0.2);
        let contradiction_penalty = (self.contradictions.len() as f32 * 0.1).min(0.3);
        let gap_penalty = (self.coverage_gaps.len() as f32 * 0.05).min(0.2);

        self.confidence =
            (avg_confidence + source_bonus - contradiction_penalty - gap_penalty).clamp(0.0, 1.0);
    }

    /// Add a contradiction
    pub fn add_contradiction(&mut self, description: &str) {
        self.contradictions.push(description.to_string());
        self.recalculate_confidence();
    }

    /// Add a coverage gap
    pub fn add_coverage_gap(&mut self, gap: &str) {
        self.coverage_gaps.push(gap.to_string());
        self.recalculate_confidence();
    }

    /// Set explainability text
    pub fn set_explanation(&mut self, why: &str) {
        self.why_explanation = Some(why.to_string());
    }

    /// Generate automatic explanation based on facts
    pub fn generate_explanation(&mut self) {
        let mut parts = Vec::new();

        parts.push(format!(
            "This incident was compiled from {} facts across {} sources.",
            self.facts.len(),
            self.coverage_scope.len()
        ));

        if !self.coverage_scope.is_empty() {
            parts.push(format!("Data sources: {}.", self.coverage_scope.join(", ")));
        }

        if !self.contradictions.is_empty() {
            parts.push(format!(
                "⚠ {} contradiction(s) detected: {}.",
                self.contradictions.len(),
                self.contradictions.join("; ")
            ));
        }

        if !self.coverage_gaps.is_empty() {
            parts.push(format!(
                "Missing coverage: {}.",
                self.coverage_gaps.join(", ")
            ));
        }

        parts.push(format!(
            "Overall confidence: {:.0}%.",
            self.confidence * 100.0
        ));

        self.why_explanation = Some(parts.join(" "));
    }
}

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Medium
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
            Severity::Informational => write!(f, "Informational"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    New,
    InProgress,
    PendingReview,
    Resolved,
    FalsePositive,
    Closed,
}

impl Default for IncidentStatus {
    fn default() -> Self {
        IncidentStatus::New
    }
}

// ============================================================================
// Builder Pattern
// ============================================================================

pub struct IncidentBuilder {
    incident: Incident,
}

impl IncidentBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            incident: Incident::new(title, "", Severity::Medium),
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.incident.description = desc.to_string();
        self
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.incident.severity = severity;
        self
    }

    pub fn add_fact(mut self, fact: Fact) -> Self {
        self.incident.add_fact(fact);
        self
    }

    pub fn mitre_technique(mut self, technique: &str) -> Self {
        self.incident.mitre_techniques.push(technique.to_string());
        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.incident.hosts.push(host.to_string());
        self
    }

    pub fn user(mut self, user: &str) -> Self {
        self.incident.users.push(user.to_string());
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.incident.tags.push(tag.to_string());
        self
    }

    pub fn build(mut self) -> Incident {
        self.incident.generate_explanation();
        self.incident
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_creation() {
        let fact = Fact::new(
            "process_creation",
            "Suspicious process spawned",
            "CrowdStrike",
            0.8,
        );
        assert_eq!(fact.primitive, "process_creation");
        assert_eq!(fact.confidence, 0.8);
    }

    #[test]
    fn test_incident_confidence_calculation() {
        let mut incident = Incident::new("Test Incident", "Test", Severity::High);

        incident.add_fact(Fact::new("process", "Fact 1", "CrowdStrike", 0.8));
        incident.add_fact(Fact::new("network", "Fact 2", "SentinelOne", 0.7));

        // Should have bonus for 2 sources
        assert!(incident.confidence > 0.7);
        assert!(incident.coverage_scope.len() == 2);
    }

    #[test]
    fn test_incident_builder() {
        let incident = IncidentBuilder::new("Ransomware Detection")
            .description("Potential ransomware activity detected")
            .severity(Severity::Critical)
            .add_fact(Fact::new(
                "file_encryption",
                "Mass file encryption detected",
                "Defender",
                0.9,
            ))
            .mitre_technique("T1486")
            .host("WORKSTATION-01")
            .build();

        assert_eq!(incident.title, "Ransomware Detection");
        assert_eq!(incident.severity, Severity::Critical);
        assert!(incident.why_explanation.is_some());
    }
}
