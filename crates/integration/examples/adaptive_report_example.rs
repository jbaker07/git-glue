//! Example usage: load manifest, generate adaptive report

use chrono::Utc;
use integration::adapter::NormalizedEvent;
use integration::adaptive_report::{generate_adaptive_report, ReportFormat};
use integration::capabilities::CapabilitiesManifest;
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    // Load manifest
    let manifest = CapabilitiesManifest::load_from_file("examples/capabilities_manifest.yaml")?;
    // Dummy coverage registry (not used in this example)
    let coverage = Default::default();

    // Create sample normalized events
    let events = vec![
        NormalizedEvent {
            id: uuid::Uuid::new_v4().to_string(),
            source: "CrowdStrike".to_string(),
            event_type: "ProcessStart".to_string(),
            timestamp: Utc::now(),
            host: "workstation-1".to_string(),
            user: Some("admin".to_string()),
            severity: Some("Medium".to_string()),
            description: Some("PowerShell execution detected".to_string()),
            raw: serde_json::json!({"cmd": "powershell.exe -enc"}),
            process: None,
            network: None,
            file: None,
            custom_fields: HashMap::new(),
        },
        NormalizedEvent {
            id: uuid::Uuid::new_v4().to_string(),
            source: "SentinelOne".to_string(),
            event_type: "ProcessStart".to_string(),
            timestamp: Utc::now(),
            host: "workstation-1".to_string(),
            user: Some("admin".to_string()),
            severity: Some("High".to_string()),
            description: Some("Encoded PowerShell detected".to_string()),
            raw: serde_json::json!({"cmd": "powershell.exe -encodedCommand"}),
            process: None,
            network: None,
            file: None,
            custom_fields: HashMap::new(),
        },
    ];

    // Generate adaptive report
    let report = generate_adaptive_report(&events, &manifest, &coverage, ReportFormat::Plain, None);

    // Print report sections
    for section in report.sections {
        println!(
            "Section: {}\nConfidence: {}\nSources: {:?}\nMissing: {:?}\nContent: {}\n---",
            section.title,
            section.confidence,
            section.sources,
            section.missing_inputs,
            section.content
        );
    }
    Ok(())
}
