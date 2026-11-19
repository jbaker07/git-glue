//! Production-ready adaptive report generation based on capabilities and coverage.

use crate::adapter::NormalizedEvent;
use crate::capabilities::{CapabilitiesManifest, CoverageRegistry};
use crate::correlation::CorrelationResult;
use crate::fact::Incident;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Report Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveReport {
    /// Report title
    pub title: String,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Report format
    pub format: ReportFormat,
    /// Report sections
    pub sections: Vec<ReportSection>,
    /// Overall confidence score
    pub overall_confidence: f32,
    /// Sources contributing to this report
    pub sources: Vec<String>,
    /// Warnings and caveats
    pub warnings: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Section title
    pub title: String,
    /// Section content (in the report format)
    pub content: String,
    /// Confidence for this section
    pub confidence: f32,
    /// Sources used for this section
    pub sources: Vec<String>,
    /// What data was missing/unavailable
    pub missing_inputs: Vec<String>,
    /// Section priority (for ordering)
    pub priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Markdown,
    Html,
    Plain,
    Json,
}

impl Default for ReportFormat {
    fn default() -> Self {
        ReportFormat::Markdown
    }
}

// ============================================================================
// Report Builder
// ============================================================================

pub struct ReportBuilder {
    title: String,
    format: ReportFormat,
    incident: Option<Incident>,
    events: Vec<NormalizedEvent>,
    manifest: Option<CapabilitiesManifest>,
    correlation: Option<CorrelationResult>,
    custom_sections: Vec<ReportSection>,
    template: Option<String>,
}

impl ReportBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            format: ReportFormat::Markdown,
            incident: None,
            events: Vec::new(),
            manifest: None,
            correlation: None,
            custom_sections: Vec::new(),
            template: None,
        }
    }

    pub fn format(mut self, format: ReportFormat) -> Self {
        self.format = format;
        self
    }

    pub fn incident(mut self, incident: Incident) -> Self {
        self.incident = Some(incident);
        self
    }

    pub fn events(mut self, events: Vec<NormalizedEvent>) -> Self {
        self.events = events;
        self
    }

    pub fn manifest(mut self, manifest: CapabilitiesManifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn correlation(mut self, result: CorrelationResult) -> Self {
        self.correlation = Some(result);
        self
    }

    pub fn add_section(mut self, section: ReportSection) -> Self {
        self.custom_sections.push(section);
        self
    }

    pub fn template(mut self, template: &str) -> Self {
        self.template = Some(template.to_string());
        self
    }

    pub fn build(self) -> AdaptiveReport {
        let mut sections = Vec::new();
        let mut sources = Vec::new();
        let mut warnings = Vec::new();

        // Executive Summary
        if let Some(ref incident) = self.incident {
            sources.extend(incident.coverage_scope.clone());
            sections.push(self.build_executive_summary(incident));
        }

        // Timeline Section
        if !self.events.is_empty() {
            sections.push(self.build_timeline_section(&self.events));
        }

        // Endpoint Analysis (if EDR data present)
        let edr_sources = [
            "CrowdStrike",
            "SentinelOne",
            "Defender",
            "Carbon Black",
            "Cortex XDR",
        ];
        let has_edr = self
            .events
            .iter()
            .any(|e| edr_sources.iter().any(|s| e.source.contains(s)));
        if has_edr {
            sections.push(self.build_endpoint_section(&self.events));
        } else {
            warnings.push("No EDR data available - endpoint analysis limited".to_string());
        }

        // Network Analysis
        let has_network = self.events.iter().any(|e| {
            e.event_type.to_lowercase().contains("network")
                || e.network
                    .as_ref()
                    .map(|n| n.dest_ip.is_some())
                    .unwrap_or(false)
        });
        if has_network {
            sections.push(self.build_network_section(&self.events));
        }

        // Identity Analysis
        let has_identity = self.events.iter().any(|e| e.user.is_some());
        if has_identity {
            sections.push(self.build_identity_section(&self.events));
        }

        // Correlation Analysis
        if let Some(ref corr) = self.correlation {
            sections.push(self.build_correlation_section(corr));
        }

        // Coverage Gaps
        if let Some(ref manifest) = self.manifest {
            let observed_sources: std::collections::HashSet<_> =
                self.events.iter().map(|e| &e.source).collect();
            let missing: Vec<_> = manifest
                .sources
                .iter()
                .filter(|s| !observed_sources.contains(s))
                .cloned()
                .collect();

            if !missing.is_empty() {
                sections.push(ReportSection {
                    title: "Coverage Gaps".to_string(),
                    content: self.format_coverage_gaps(&missing),
                    confidence: 0.5,
                    sources: Vec::new(),
                    missing_inputs: missing.clone(),
                    priority: 90,
                });
                warnings.push(format!("Missing data from: {}", missing.join(", ")));
            }
        }

        // Custom sections
        sections.extend(self.custom_sections);

        // Sort by priority
        sections.sort_by_key(|s| s.priority);

        // Calculate overall confidence
        let overall_confidence = if sections.is_empty() {
            0.0
        } else {
            sections.iter().map(|s| s.confidence).sum::<f32>() / sections.len() as f32
        };

        AdaptiveReport {
            title: self.title,
            generated_at: Utc::now(),
            format: self.format,
            sections,
            overall_confidence,
            sources,
            warnings,
            metadata: HashMap::new(),
        }
    }

    fn build_executive_summary(&self, incident: &Incident) -> ReportSection {
        let content = match self.format {
            ReportFormat::Markdown => format!(
                "## Executive Summary\n\n\
                **Incident:** {}\n\n\
                **Severity:** {:?}\n\n\
                **Confidence:** {:.0}%\n\n\
                **Status:** {:?}\n\n\
                {}\n\n\
                ### Key Findings\n\n\
                - {} supporting facts from {} sources\n\
                - {} hosts affected\n\
                - {} users involved\n\
                {}",
                incident.title,
                incident.severity,
                incident.confidence * 100.0,
                incident.status,
                incident.description,
                incident.facts.len(),
                incident.coverage_scope.len(),
                incident.hosts.len(),
                incident.users.len(),
                if !incident.mitre_techniques.is_empty() {
                    format!("- MITRE ATT&CK: {}", incident.mitre_techniques.join(", "))
                } else {
                    String::new()
                }
            ),
            ReportFormat::Html => format!(
                "<h2>Executive Summary</h2>\
                <p><strong>Incident:</strong> {}</p>\
                <p><strong>Severity:</strong> {:?}</p>\
                <p><strong>Confidence:</strong> {:.0}%</p>\
                <p>{}</p>",
                incident.title,
                incident.severity,
                incident.confidence * 100.0,
                incident.description
            ),
            ReportFormat::Plain => format!(
                "EXECUTIVE SUMMARY\n\
                Incident: {}\n\
                Severity: {:?}\n\
                Confidence: {:.0}%\n\
                {}",
                incident.title,
                incident.severity,
                incident.confidence * 100.0,
                incident.description
            ),
            ReportFormat::Json => serde_json::to_string_pretty(incident).unwrap_or_default(),
        };

        ReportSection {
            title: "Executive Summary".to_string(),
            content,
            confidence: incident.confidence,
            sources: incident.coverage_scope.clone(),
            missing_inputs: incident.coverage_gaps.clone(),
            priority: 10,
        }
    }

    fn build_timeline_section(&self, events: &[NormalizedEvent]) -> ReportSection {
        let mut sorted_events: Vec<_> = events.iter().collect();
        sorted_events.sort_by_key(|e| e.timestamp);

        let content = match self.format {
            ReportFormat::Markdown => {
                let mut md = "## Timeline\n\n".to_string();
                for event in sorted_events.iter().take(50) {
                    md.push_str(&format!(
                        "- **{}** [{}] {} on `{}`{}\n",
                        event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        event.source,
                        event.event_type,
                        event.host,
                        event
                            .user
                            .as_ref()
                            .map(|u| format!(" (user: {})", u))
                            .unwrap_or_default()
                    ));
                }
                if events.len() > 50 {
                    md.push_str(&format!("\n*... and {} more events*\n", events.len() - 50));
                }
                md
            }
            ReportFormat::Html => {
                let mut html = "<h2>Timeline</h2><ul>".to_string();
                for event in sorted_events.iter().take(50) {
                    html.push_str(&format!(
                        "<li><strong>{}</strong> [{}] {} on <code>{}</code></li>",
                        event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        event.source,
                        event.event_type,
                        event.host
                    ));
                }
                html.push_str("</ul>");
                html
            }
            _ => format!(
                "Timeline: {} events from {} to {}",
                events.len(),
                sorted_events
                    .first()
                    .map(|e| e.timestamp.to_string())
                    .unwrap_or_default(),
                sorted_events
                    .last()
                    .map(|e| e.timestamp.to_string())
                    .unwrap_or_default()
            ),
        };

        let sources: Vec<_> = events
            .iter()
            .map(|e| e.source.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        ReportSection {
            title: "Timeline".to_string(),
            content,
            confidence: 0.9,
            sources,
            missing_inputs: Vec::new(),
            priority: 20,
        }
    }

    fn build_endpoint_section(&self, events: &[NormalizedEvent]) -> ReportSection {
        let process_events: Vec<_> = events.iter().filter(|e| e.process.is_some()).collect();

        let content = match self.format {
            ReportFormat::Markdown => {
                let mut md = "## Endpoint Analysis\n\n### Process Activity\n\n".to_string();
                for event in process_events.iter().take(20) {
                    if let Some(proc) = &event.process {
                        md.push_str(&format!(
                            "- `{}`: {}{}\n",
                            proc.name.as_deref().unwrap_or("unknown"),
                            proc.command_line.as_deref().unwrap_or("(no cmdline)"),
                            event
                                .severity
                                .as_ref()
                                .map(|s| format!(" [{}]", s))
                                .unwrap_or_default()
                        ));
                    }
                }
                md
            }
            _ => format!(
                "Endpoint Analysis: {} process events analyzed",
                process_events.len()
            ),
        };

        ReportSection {
            title: "Endpoint Analysis".to_string(),
            content,
            confidence: 0.85,
            sources: process_events
                .iter()
                .map(|e| e.source.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
            missing_inputs: Vec::new(),
            priority: 30,
        }
    }

    fn build_network_section(&self, events: &[NormalizedEvent]) -> ReportSection {
        let network_events: Vec<_> = events.iter().filter(|e| e.network.is_some()).collect();

        let content = match self.format {
            ReportFormat::Markdown => {
                let mut md = "## Network Analysis\n\n".to_string();
                for event in network_events.iter().take(20) {
                    if let Some(net) = &event.network {
                        md.push_str(&format!(
                            "- {} -> {}:{} ({})\n",
                            event.host,
                            net.dest_ip.as_deref().unwrap_or("?"),
                            net.dest_port.unwrap_or(0),
                            net.protocol.as_deref().unwrap_or("tcp")
                        ));
                    }
                }
                md
            }
            _ => format!("Network Analysis: {} network events", network_events.len()),
        };

        ReportSection {
            title: "Network Analysis".to_string(),
            content,
            confidence: 0.8,
            sources: network_events
                .iter()
                .map(|e| e.source.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
            missing_inputs: Vec::new(),
            priority: 40,
        }
    }

    fn build_identity_section(&self, events: &[NormalizedEvent]) -> ReportSection {
        let users: std::collections::HashSet<_> =
            events.iter().filter_map(|e| e.user.clone()).collect();

        let content = match self.format {
            ReportFormat::Markdown => {
                let mut md = "## Identity Analysis\n\n### Users Involved\n\n".to_string();
                for user in &users {
                    let user_events: Vec<_> = events
                        .iter()
                        .filter(|e| e.user.as_ref() == Some(user))
                        .collect();
                    md.push_str(&format!(
                        "- **{}**: {} events across {} hosts\n",
                        user,
                        user_events.len(),
                        user_events
                            .iter()
                            .map(|e| &e.host)
                            .collect::<std::collections::HashSet<_>>()
                            .len()
                    ));
                }
                md
            }
            _ => format!("Identity Analysis: {} unique users", users.len()),
        };

        ReportSection {
            title: "Identity Analysis".to_string(),
            content,
            confidence: 0.75,
            sources: Vec::new(),
            missing_inputs: Vec::new(),
            priority: 50,
        }
    }

    fn build_correlation_section(&self, corr: &CorrelationResult) -> ReportSection {
        let content = match self.format {
            ReportFormat::Markdown => {
                let mut md = format!(
                    "## Correlation Analysis\n\n\
                    **Overall Correlation Confidence:** {:.0}%\n\n",
                    corr.confidence * 100.0
                );

                if !corr.corroborations.is_empty() {
                    md.push_str("### Corroborated Findings\n\n");
                    for c in &corr.corroborations {
                        md.push_str(&format!(
                            "- **{}**: Confirmed by {} ({:.0}% combined confidence)\n",
                            c.subject,
                            c.agreeing_sources.join(", "),
                            c.combined_confidence * 100.0
                        ));
                    }
                    md.push('\n');
                }

                if !corr.contradictions.is_empty() {
                    md.push_str("### Contradictions Detected\n\n");
                    for c in &corr.contradictions {
                        md.push_str(&format!(
                            "- ⚠️ **{}** ({:?}): {}\n",
                            c.subject, c.severity, c.description
                        ));
                    }
                    md.push('\n');
                }

                if !corr.coverage_gaps.is_empty() {
                    md.push_str("### Coverage Gaps\n\n");
                    for g in &corr.coverage_gaps {
                        md.push_str(&format!(
                            "- Missing: {} ({:?} impact)\n",
                            g.missing, g.impact
                        ));
                    }
                }

                md
            }
            _ => corr.explanation.clone(),
        };

        ReportSection {
            title: "Correlation Analysis".to_string(),
            content,
            confidence: corr.confidence,
            sources: Vec::new(),
            missing_inputs: corr
                .coverage_gaps
                .iter()
                .map(|g| g.missing.clone())
                .collect(),
            priority: 60,
        }
    }

    fn format_coverage_gaps(&self, missing: &[String]) -> String {
        match self.format {
            ReportFormat::Markdown => {
                let mut md =
                    "## Coverage Gaps\n\n⚠️ **The following data sources are not available:**\n\n"
                        .to_string();
                for source in missing {
                    md.push_str(&format!("- {}\n", source));
                }
                md.push_str("\nThis may limit the completeness of the analysis.\n");
                md
            }
            ReportFormat::Html => {
                format!(
                    "<h2>Coverage Gaps</h2><p>Missing sources: {}</p>",
                    missing.join(", ")
                )
            }
            _ => format!("Coverage Gaps: Missing {}", missing.join(", ")),
        }
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Generate an adaptive report from events and manifest
pub fn generate_adaptive_report(
    events: &[NormalizedEvent],
    manifest: &CapabilitiesManifest,
    _coverage: &CoverageRegistry,
    format: ReportFormat,
    _template: Option<&str>,
) -> AdaptiveReport {
    ReportBuilder::new("Incident Analysis Report")
        .format(format)
        .events(events.to_vec())
        .manifest(manifest.clone())
        .build()
}

/// Generate a report from an incident
pub fn generate_incident_report(incident: &Incident, format: ReportFormat) -> AdaptiveReport {
    ReportBuilder::new(&format!("Incident Report: {}", incident.title))
        .format(format)
        .incident(incident.clone())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_builder() {
        let report = ReportBuilder::new("Test Report")
            .format(ReportFormat::Markdown)
            .build();

        assert_eq!(report.title, "Test Report");
        assert_eq!(report.format, ReportFormat::Markdown);
    }
}
