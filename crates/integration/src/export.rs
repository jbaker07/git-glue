//! Production-ready export module: export incidents, reports, and bundles.

use crate::adapter::NormalizedEvent;
use crate::adaptive_report::AdaptiveReport;
use crate::fact::Incident;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

// ============================================================================
// Export Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Output format
    pub format: ExportFormat,
    /// Include raw events
    pub include_raw_events: bool,
    /// Include metadata
    pub include_metadata: bool,
    /// Pretty print JSON
    pub pretty_print: bool,
    /// Compress output
    pub compress: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_raw_events: false,
            include_metadata: true,
            pretty_print: true,
            compress: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Yaml,
    Csv,
    Html,
    Markdown,
    Pdf,
}

// ============================================================================
// Export Bundle
// ============================================================================

/// A complete export bundle containing all incident data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportBundle {
    /// Bundle metadata
    pub metadata: BundleMetadata,
    /// The incident
    pub incident: Option<Incident>,
    /// Normalized events
    pub events: Vec<NormalizedEvent>,
    /// Generated report (if any)
    pub report: Option<AdaptiveReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    pub version: String,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub exporter: String,
    pub source_count: usize,
    pub event_count: usize,
}

impl ExportBundle {
    pub fn new() -> Self {
        Self {
            metadata: BundleMetadata {
                version: "1.0".to_string(),
                exported_at: chrono::Utc::now(),
                exporter: "integration".to_string(),
                source_count: 0,
                event_count: 0,
            },
            incident: None,
            events: Vec::new(),
            report: None,
        }
    }

    pub fn with_incident(mut self, incident: Incident) -> Self {
        self.metadata.source_count = incident.coverage_scope.len();
        self.incident = Some(incident);
        self
    }

    pub fn with_events(mut self, events: Vec<NormalizedEvent>) -> Self {
        self.metadata.event_count = events.len();
        self.events = events;
        self
    }

    pub fn with_report(mut self, report: AdaptiveReport) -> Self {
        self.report = Some(report);
        self
    }
}

impl Default for ExportBundle {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Exporter
// ============================================================================

pub struct Exporter {
    config: ExportConfig,
}

impl Exporter {
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(ExportConfig::default())
    }

    /// Export a bundle to a file
    pub fn export_bundle(&self, bundle: &ExportBundle, path: &str) -> Result<()> {
        let content = self.render_bundle(bundle)?;
        let mut file = File::create(path)
            .with_context(|| format!("Failed to create export file: {}", path))?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Export an incident to a file
    pub fn export_incident(&self, incident: &Incident, path: &str) -> Result<()> {
        let bundle = ExportBundle::new().with_incident(incident.clone());
        self.export_bundle(&bundle, path)
    }

    /// Export events to a file
    pub fn export_events(&self, events: &[NormalizedEvent], path: &str) -> Result<()> {
        let bundle = ExportBundle::new().with_events(events.to_vec());
        self.export_bundle(&bundle, path)
    }

    /// Export a report to a file
    pub fn export_report(&self, report: &AdaptiveReport, path: &str) -> Result<()> {
        match self.config.format {
            ExportFormat::Pdf => {
                crate::export_pdf::export_report_to_pdf(report, path)?;
            }
            ExportFormat::Html => {
                let html = self.render_report_html(report);
                std::fs::write(path, html)?;
            }
            ExportFormat::Markdown => {
                let md = self.render_report_markdown(report);
                std::fs::write(path, md)?;
            }
            _ => {
                let content = if self.config.pretty_print {
                    serde_json::to_string_pretty(report)?
                } else {
                    serde_json::to_string(report)?
                };
                std::fs::write(path, content)?;
            }
        }
        Ok(())
    }

    fn render_bundle(&self, bundle: &ExportBundle) -> Result<String> {
        match self.config.format {
            ExportFormat::Json => {
                if self.config.pretty_print {
                    Ok(serde_json::to_string_pretty(bundle)?)
                } else {
                    Ok(serde_json::to_string(bundle)?)
                }
            }
            ExportFormat::Yaml => Ok(serde_yaml::to_string(bundle)?),
            ExportFormat::Csv => self.render_events_csv(&bundle.events),
            ExportFormat::Markdown => Ok(self.render_bundle_markdown(bundle)),
            ExportFormat::Html => Ok(self.render_bundle_html(bundle)),
            ExportFormat::Pdf => {
                anyhow::bail!("PDF export requires using export_report() with a report");
            }
        }
    }

    fn render_events_csv(&self, events: &[NormalizedEvent]) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("id,source,event_type,timestamp,host,user,severity\n");

        for event in events {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                event.id,
                event.source,
                event.event_type,
                event.timestamp.to_rfc3339(),
                event.host,
                event.user.as_deref().unwrap_or(""),
                event.severity.as_deref().unwrap_or("")
            ));
        }

        Ok(csv)
    }

    fn render_bundle_markdown(&self, bundle: &ExportBundle) -> String {
        let mut md = String::new();

        // Metadata
        md.push_str(&format!("# Export Bundle\n\n"));
        md.push_str(&format!(
            "- **Exported:** {}\n",
            bundle.metadata.exported_at
        ));
        md.push_str(&format!("- **Events:** {}\n", bundle.metadata.event_count));
        md.push_str(&format!(
            "- **Sources:** {}\n\n",
            bundle.metadata.source_count
        ));

        // Incident
        if let Some(incident) = &bundle.incident {
            md.push_str(&format!("## Incident: {}\n\n", incident.title));
            md.push_str(&format!("**Severity:** {:?}\n\n", incident.severity));
            md.push_str(&format!(
                "**Confidence:** {:.0}%\n\n",
                incident.confidence * 100.0
            ));
            md.push_str(&format!("{}\n\n", incident.description));
        }

        // Events summary
        if !bundle.events.is_empty() {
            md.push_str("## Events\n\n");
            md.push_str("| Time | Source | Type | Host |\n");
            md.push_str("|------|--------|------|------|\n");
            for event in bundle.events.iter().take(50) {
                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    event.source,
                    event.event_type,
                    event.host
                ));
            }
            if bundle.events.len() > 50 {
                md.push_str(&format!(
                    "\n*... and {} more events*\n",
                    bundle.events.len() - 50
                ));
            }
        }

        md
    }

    fn render_bundle_html(&self, bundle: &ExportBundle) -> String {
        let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<title>Export Bundle</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #f4f4f4; }\n");
        html.push_str(".severity-critical { color: #d32f2f; }\n");
        html.push_str(".severity-high { color: #f57c00; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        html.push_str("<h1>Export Bundle</h1>\n");
        html.push_str(&format!(
            "<p><strong>Exported:</strong> {}</p>\n",
            bundle.metadata.exported_at
        ));
        html.push_str(&format!(
            "<p><strong>Events:</strong> {}</p>\n",
            bundle.metadata.event_count
        ));

        if let Some(incident) = &bundle.incident {
            html.push_str(&format!("<h2>Incident: {}</h2>\n", incident.title));
            html.push_str(&format!(
                "<p><strong>Severity:</strong> {:?}</p>\n",
                incident.severity
            ));
            html.push_str(&format!("<p>{}</p>\n", incident.description));
        }

        if !bundle.events.is_empty() {
            html.push_str("<h2>Events</h2>\n");
            html.push_str(
                "<table>\n<tr><th>Time</th><th>Source</th><th>Type</th><th>Host</th></tr>\n",
            );
            for event in bundle.events.iter().take(100) {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    event.source,
                    event.event_type,
                    event.host
                ));
            }
            html.push_str("</table>\n");
        }

        html.push_str("</body>\n</html>");
        html
    }

    fn render_report_markdown(&self, report: &AdaptiveReport) -> String {
        let mut md = format!("# {}\n\n", report.title);
        md.push_str(&format!("*Generated: {}*\n\n", report.generated_at));
        md.push_str(&format!(
            "**Overall Confidence:** {:.0}%\n\n",
            report.overall_confidence * 100.0
        ));

        for section in &report.sections {
            md.push_str(&section.content);
            md.push_str("\n\n");
        }

        if !report.warnings.is_empty() {
            md.push_str("---\n\n## Warnings\n\n");
            for warning in &report.warnings {
                md.push_str(&format!("- ⚠️ {}\n", warning));
            }
        }

        md
    }

    fn render_report_html(&self, report: &AdaptiveReport) -> String {
        let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str(&format!("<title>{}</title>\n", report.title));
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; max-width: 900px; }\n");
        html.push_str("h1 { color: #333; }\n");
        html.push_str("h2 { color: #555; border-bottom: 1px solid #eee; padding-bottom: 10px; }\n");
        html.push_str(".warning { color: #f57c00; }\n");
        html.push_str(".meta { color: #666; font-size: 0.9em; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        html.push_str(&format!("<h1>{}</h1>\n", report.title));
        html.push_str(&format!(
            "<p class=\"meta\">Generated: {} | Confidence: {:.0}%</p>\n",
            report.generated_at,
            report.overall_confidence * 100.0
        ));

        for section in &report.sections {
            // Convert markdown-style content to basic HTML
            let content = section
                .content
                .replace("## ", "<h2>")
                .replace("### ", "<h3>")
                .replace("\n\n", "</p><p>")
                .replace("**", "<strong>")
                .replace("*", "<em>");
            html.push_str(&format!("<div>{}</div>\n", content));
        }

        if !report.warnings.is_empty() {
            html.push_str("<hr>\n<h2>Warnings</h2>\n<ul class=\"warning\">\n");
            for warning in &report.warnings {
                html.push_str(&format!("<li>{}</li>\n", warning));
            }
            html.push_str("</ul>\n");
        }

        html.push_str("</body>\n</html>");
        html
    }
}

impl Default for Exporter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Export events to a file (backward compatible)
pub fn export_to_file(events: &[serde_json::Value], path: &str) -> Result<()> {
    let data = serde_json::to_string_pretty(events)?;
    std::fs::write(path, data)?;
    Ok(())
}

/// Export normalized events to JSON
pub fn export_events_json(events: &[NormalizedEvent], path: &str) -> Result<()> {
    Exporter::with_defaults().export_events(events, path)
}

/// Export incident to JSON
pub fn export_incident_json(incident: &Incident, path: &str) -> Result<()> {
    Exporter::with_defaults().export_incident(incident, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert_eq!(config.format, ExportFormat::Json);
        assert!(config.pretty_print);
    }

    #[test]
    fn test_bundle_creation() {
        let bundle = ExportBundle::new();
        assert_eq!(bundle.metadata.version, "1.0");
    }
}
