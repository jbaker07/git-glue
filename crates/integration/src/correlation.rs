//! Production-ready correlation engine: detect contradictions, gaps, and generate explanations.

use crate::adapter::NormalizedEvent;
use crate::capabilities::{CapabilitiesManifest, CoverageRegistry};
use crate::fact::Incident;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;

// ============================================================================
// Correlation Result
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    /// Detected contradictions
    pub contradictions: Vec<Contradiction>,
    /// Coverage gaps
    pub coverage_gaps: Vec<CoverageGap>,
    /// Corroborated facts (multiple sources agree)
    pub corroborations: Vec<Corroboration>,
    /// Overall correlation confidence
    pub confidence: f32,
    /// Summary explanation
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    /// What is being contradicted
    pub subject: String,
    /// Sources involved
    pub sources: Vec<String>,
    /// Nature of the contradiction
    pub description: String,
    /// Severity of the contradiction
    pub severity: ContradictionSeverity,
    /// Suggested resolution
    pub resolution_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContradictionSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGap {
    /// What primitive/capability is missing
    pub missing: String,
    /// Which source(s) could provide it
    pub potential_sources: Vec<String>,
    /// Impact on incident confidence
    pub impact: GapImpact,
    /// Explanation
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapImpact {
    Critical,
    Significant,
    Minor,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corroboration {
    /// The fact/assertion being corroborated
    pub subject: String,
    /// Sources that agree
    pub agreeing_sources: Vec<String>,
    /// Combined confidence
    pub combined_confidence: f32,
}

// ============================================================================
// Correlation Engine
// ============================================================================

pub struct CorrelationEngine {
    manifest: Option<CapabilitiesManifest>,
    registry: CoverageRegistry,
    /// Time window for event correlation (in seconds)
    time_window_secs: i64,
    /// Minimum sources for high confidence
    min_sources_high_confidence: usize,
}

impl CorrelationEngine {
    pub fn new() -> Self {
        Self {
            manifest: None,
            registry: CoverageRegistry::new(),
            time_window_secs: 300, // 5 minutes
            min_sources_high_confidence: 2,
        }
    }

    pub fn with_manifest(mut self, manifest: CapabilitiesManifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn with_time_window(mut self, secs: i64) -> Self {
        self.time_window_secs = secs;
        self
    }

    /// Correlate events from multiple sources
    pub fn correlate_events(&mut self, events: &[NormalizedEvent]) -> CorrelationResult {
        let mut contradictions = Vec::new();
        let mut corroborations = Vec::new();

        // Group events by host and time window
        let grouped = Self::group_events_static(events, self.time_window_secs);

        // Analyze each group for contradictions and corroborations
        for (key, group_events) in &grouped {
            debug!("Analyzing group {} with {} events", key, group_events.len());

            // Check for contradictions within group
            Self::detect_contradictions_static(group_events, &mut contradictions);

            // Check for corroborations
            Self::detect_corroborations_static(
                group_events,
                self.min_sources_high_confidence,
                &mut corroborations,
            );
        }

        // Update registry after analysis
        for event in events {
            self.registry
                .record_observation(&event.event_type, &event.source);
        }

        // Detect coverage gaps
        let coverage_gaps = self.detect_coverage_gaps(events);

        // Calculate overall confidence
        let confidence =
            self.calculate_correlation_confidence(&contradictions, &corroborations, &coverage_gaps);

        // Generate explanation
        let explanation =
            self.generate_explanation(&contradictions, &corroborations, &coverage_gaps);

        CorrelationResult {
            contradictions,
            coverage_gaps,
            corroborations,
            confidence,
            explanation,
        }
    }

    /// Analyze and enrich an incident
    pub fn analyze_incident(&mut self, incident: &mut Incident) {
        // Analyze facts for contradictions
        let mut fact_sources: HashMap<String, Vec<(String, f32)>> = HashMap::new();

        for fact in &incident.facts {
            fact_sources
                .entry(fact.primitive.clone())
                .or_default()
                .push((fact.source.clone(), fact.confidence));
        }

        // Check for contradictions in same primitive from different sources
        for (primitive, sources) in &fact_sources {
            if sources.len() > 1 {
                let confidences: Vec<f32> = sources.iter().map(|(_, c)| *c).collect();
                let max_conf = confidences.iter().cloned().fold(0.0_f32, f32::max);
                let min_conf = confidences.iter().cloned().fold(1.0_f32, f32::min);

                // Significant confidence difference indicates potential contradiction
                if max_conf - min_conf > 0.3 {
                    let source_names: Vec<String> =
                        sources.iter().map(|(s, _)| s.clone()).collect();
                    incident.add_contradiction(&format!(
                        "Confidence divergence for '{}': sources {} report significantly different confidence levels ({:.0}% vs {:.0}%)",
                        primitive,
                        source_names.join(", "),
                        max_conf * 100.0,
                        min_conf * 100.0
                    ));
                }
            }
        }

        // Check for coverage gaps based on manifest
        if let Some(manifest) = &self.manifest {
            let covered_primitives: HashSet<String> =
                incident.facts.iter().map(|f| f.primitive.clone()).collect();
            let all_primitives = manifest.all_primitives();

            for primitive in &all_primitives {
                if !covered_primitives.contains(primitive) {
                    let potential_sources = manifest.sources_for_primitive(primitive);
                    if !potential_sources.is_empty() {
                        incident.add_coverage_gap(&format!(
                            "Missing '{}' telemetry (available from: {})",
                            primitive,
                            potential_sources.join(", ")
                        ));
                    }
                }
            }
        }

        // Generate explanation
        incident.generate_explanation();
    }

    fn group_events_static(
        events: &[NormalizedEvent],
        time_window_secs: i64,
    ) -> HashMap<String, Vec<&NormalizedEvent>> {
        let mut groups: HashMap<String, Vec<&NormalizedEvent>> = HashMap::new();

        for event in events {
            // Group by host + time window
            let time_bucket = event.timestamp.timestamp() / time_window_secs;
            let key = format!("{}_{}", event.host, time_bucket);
            groups.entry(key).or_default().push(event);
        }

        groups
    }

    fn detect_contradictions_static(
        events: &[&NormalizedEvent],
        contradictions: &mut Vec<Contradiction>,
    ) {
        // Group by event type within this time window
        let mut by_type: HashMap<String, Vec<&NormalizedEvent>> = HashMap::new();
        for event in events {
            by_type
                .entry(event.event_type.clone())
                .or_default()
                .push(event);
        }

        for (event_type, type_events) in &by_type {
            if type_events.len() < 2 {
                continue;
            }

            // Check for contradictory severities
            let severities: HashSet<String> = type_events
                .iter()
                .filter_map(|e| e.severity.clone())
                .collect();

            if severities.len() > 1
                && (severities.contains("Critical") || severities.contains("High"))
                && (severities.contains("Low") || severities.contains("Informational"))
            {
                let sources: Vec<String> = type_events.iter().map(|e| e.source.clone()).collect();
                contradictions.push(Contradiction {
                    subject: event_type.clone(),
                    sources: sources.clone(),
                    description: format!(
                        "Severity contradiction: sources report conflicting severities ({:?})",
                        severities
                    ),
                    severity: ContradictionSeverity::High,
                    resolution_hint: Some(
                        "Review raw events and apply analyst judgment".to_string(),
                    ),
                });
            }

            // Check for process name contradictions (different processes, same time/host)
            let process_names: HashSet<String> = type_events
                .iter()
                .filter_map(|e| e.process.as_ref())
                .filter_map(|p| p.name.clone())
                .collect();

            if process_names.len() > 2 {
                let sources: Vec<String> = type_events.iter().map(|e| e.source.clone()).collect();
                contradictions.push(Contradiction {
                    subject: format!("{} process attribution", event_type),
                    sources,
                    description: format!(
                        "Process attribution conflict: multiple processes reported ({:?})",
                        process_names
                    ),
                    severity: ContradictionSeverity::Medium,
                    resolution_hint: Some(
                        "May indicate process injection or parent/child relationship".to_string(),
                    ),
                });
            }
        }
    }

    fn detect_corroborations_static(
        events: &[&NormalizedEvent],
        min_sources: usize,
        corroborations: &mut Vec<Corroboration>,
    ) {
        let mut by_type: HashMap<String, Vec<&NormalizedEvent>> = HashMap::new();
        for event in events {
            by_type
                .entry(event.event_type.clone())
                .or_default()
                .push(event);
        }

        for (event_type, type_events) in &by_type {
            let sources: HashSet<String> = type_events.iter().map(|e| e.source.clone()).collect();

            if sources.len() >= min_sources {
                let avg_confidence: f32 = type_events
                    .iter()
                    .filter_map(|e| e.severity.as_ref())
                    .map(|s| severity_to_confidence(s))
                    .sum::<f32>()
                    / type_events.len() as f32;

                // Boost confidence for multi-source corroboration
                let combined_confidence =
                    (avg_confidence + 0.1 * (sources.len() as f32 - 1.0)).min(1.0);

                corroborations.push(Corroboration {
                    subject: event_type.clone(),
                    agreeing_sources: sources.into_iter().collect(),
                    combined_confidence,
                });
            }
        }
    }

    fn detect_coverage_gaps(&self, events: &[NormalizedEvent]) -> Vec<CoverageGap> {
        let mut gaps = Vec::new();

        if let Some(manifest) = &self.manifest {
            let observed_types: HashSet<String> =
                events.iter().map(|e| e.event_type.clone()).collect();
            let observed_sources: HashSet<String> =
                events.iter().map(|e| e.source.clone()).collect();

            // Check for expected but missing event types
            let expected_types = manifest.all_event_types();
            for expected in &expected_types {
                if !observed_types.contains(expected) {
                    let potential = manifest
                        .sources
                        .iter()
                        .filter(|s| {
                            manifest
                                .capabilities
                                .get(*s)
                                .map(|c| c.event_types.contains(expected))
                                .unwrap_or(false)
                        })
                        .cloned()
                        .collect::<Vec<_>>();

                    if !potential.is_empty() {
                        gaps.push(CoverageGap {
                            missing: expected.clone(),
                            potential_sources: potential,
                            impact: GapImpact::Minor,
                            description: format!("Expected event type '{}' not observed", expected),
                        });
                    }
                }
            }

            // Check for missing sources that should be present
            for source in &manifest.sources {
                if !observed_sources.contains(source) {
                    gaps.push(CoverageGap {
                        missing: source.clone(),
                        potential_sources: vec![source.clone()],
                        impact: GapImpact::Significant,
                        description: format!("Expected source '{}' not providing data", source),
                    });
                }
            }
        }

        gaps
    }

    fn calculate_correlation_confidence(
        &self,
        contradictions: &[Contradiction],
        corroborations: &[Corroboration],
        gaps: &[CoverageGap],
    ) -> f32 {
        let mut confidence = 0.5; // Base confidence

        // Boost for corroborations
        confidence += corroborations.len() as f32 * 0.1;

        // Penalty for contradictions
        for c in contradictions {
            match c.severity {
                ContradictionSeverity::Critical => confidence -= 0.2,
                ContradictionSeverity::High => confidence -= 0.15,
                ContradictionSeverity::Medium => confidence -= 0.1,
                ContradictionSeverity::Low => confidence -= 0.05,
            }
        }

        // Penalty for gaps
        for g in gaps {
            match g.impact {
                GapImpact::Critical => confidence -= 0.15,
                GapImpact::Significant => confidence -= 0.1,
                GapImpact::Minor => confidence -= 0.05,
                GapImpact::Informational => confidence -= 0.02,
            }
        }

        confidence.clamp(0.0, 1.0)
    }

    fn generate_explanation(
        &self,
        contradictions: &[Contradiction],
        corroborations: &[Corroboration],
        gaps: &[CoverageGap],
    ) -> String {
        let mut parts = Vec::new();

        if !corroborations.is_empty() {
            parts.push(format!(
                "{} event type(s) corroborated by multiple sources",
                corroborations.len()
            ));
        }

        if !contradictions.is_empty() {
            let critical = contradictions
                .iter()
                .filter(|c| {
                    c.severity == ContradictionSeverity::Critical
                        || c.severity == ContradictionSeverity::High
                })
                .count();
            parts.push(format!(
                "{} contradiction(s) detected ({} high/critical)",
                contradictions.len(),
                critical
            ));
        }

        if !gaps.is_empty() {
            parts.push(format!("{} coverage gap(s) identified", gaps.len()));
        }

        if parts.is_empty() {
            "No significant correlation findings".to_string()
        } else {
            parts.join(". ") + "."
        }
    }
}

impl Default for CorrelationEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn severity_to_confidence(severity: &str) -> f32 {
    match severity.to_lowercase().as_str() {
        "critical" => 0.95,
        "high" => 0.85,
        "medium" => 0.7,
        "low" => 0.5,
        "informational" | "info" => 0.3,
        _ => 0.5,
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Analyze correlation across events (convenience function)
pub fn analyze_correlation(events: &[NormalizedEvent]) -> CorrelationResult {
    let mut engine = CorrelationEngine::new();
    engine.correlate_events(events)
}

/// Analyze correlation with a capabilities manifest
pub fn analyze_correlation_with_manifest(
    events: &[NormalizedEvent],
    manifest: &CapabilitiesManifest,
) -> CorrelationResult {
    let mut engine = CorrelationEngine::new().with_manifest(manifest.clone());
    engine.correlate_events(events)
}

/// Analyze and enrich an incident
pub fn analyze_incident(incident: &mut Incident, manifest: Option<&CapabilitiesManifest>) {
    let mut engine = CorrelationEngine::new();
    if let Some(m) = manifest {
        engine = engine.with_manifest(m.clone());
    }
    engine.analyze_incident(incident);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_engine() {
        let mut engine = CorrelationEngine::new();
        let result = engine.correlate_events(&[]);
        assert_eq!(result.contradictions.len(), 0);
        assert!(result.confidence > 0.0);
    }
}
