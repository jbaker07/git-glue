//! integration_metrics - Run metrics/proof harness over incidents and integrations
//!
//! Usage:
//!   integration_metrics [OPTIONS]
//!
//! Options:
//!   --incidents-dir DIR    Directory containing incident JSONL files
//!   --segments-dir DIR     Directory containing segment files (for event count)
//!   --config PATH          Integration config file (YAML/JSON)
//!   --output DIR           Output directory for metrics (default: /var/lib/edr/metrics)
//!   --per-incident         Include per-incident breakdown
//!   --export               Run export to configured sinks
//!   --ingest PATH          Run ingest from specified file
//!   --help                 Show this help

use chrono::Utc;
use edr_locald::hypothesis::{
    EvidencePtr, Fact, FactType, Incident, IncidentStatus, ScopeKey, Severity, TimelineEntry,
    TimelineEntryKind,
};
use edr_locald::integrations::{
    config::{ExportSinkConfig, ExportSinkType, IntegrationConfig},
    export::IncidentExporter,
    ingest::VendorAlertIngester,
    metrics::{IntegrationMetrics, MetricsReport},
};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    let mut incidents_dir = PathBuf::from("/var/lib/edr/incidents");
    let mut segments_dir = PathBuf::from("/var/lib/edr/segments");
    let mut output_dir = PathBuf::from("/var/lib/edr/metrics");
    let mut config_path: Option<PathBuf> = None;
    let mut per_incident = false;
    let mut run_export = false;
    let mut ingest_path: Option<PathBuf> = None;

    // Parse args
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--incidents-dir" => {
                if i + 1 < args.len() {
                    incidents_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--segments-dir" => {
                if i + 1 < args.len() {
                    segments_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--config" => {
                if i + 1 < args.len() {
                    config_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--per-incident" => {
                per_incident = true;
                i += 1;
            }
            "--export" => {
                run_export = true;
                i += 1;
            }
            "--ingest" => {
                if i + 1 < args.len() {
                    ingest_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    eprintln!("═══════════════════════════════════════════════════════════════");
    eprintln!("  Integration Metrics Harness");
    eprintln!("═══════════════════════════════════════════════════════════════");
    eprintln!("  Incidents dir: {}", incidents_dir.display());
    eprintln!("  Segments dir:  {}", segments_dir.display());
    eprintln!("  Output dir:    {}", output_dir.display());
    eprintln!("  Per-incident:  {}", per_incident);
    eprintln!("  Run export:    {}", run_export);
    if let Some(ref p) = ingest_path {
        eprintln!("  Ingest path:   {}", p.display());
    }
    eprintln!("───────────────────────────────────────────────────────────────");

    // Load configuration
    let config = config_path
        .map(|p| {
            if p.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                IntegrationConfig::from_yaml_file(&p)
            } else {
                IntegrationConfig::from_json_file(&p)
            }
        })
        .transpose()
        .unwrap_or_else(|e| {
            eprintln!("[warn] Failed to load config: {}", e);
            None
        })
        .unwrap_or_default();

    // Load incidents
    let incidents = load_incidents(&incidents_dir);
    eprintln!("[info] Loaded {} incidents", incidents.len());

    // Count raw events (approximate from segment files)
    let raw_event_count = count_raw_events(&segments_dir);
    eprintln!("[info] Counted ~{} raw events in segments", raw_event_count);

    // Run export if requested
    let export_result = if run_export && config.export.enabled && !config.export.sinks.is_empty() {
        let sink_config = &config.export.sinks[0];
        match IncidentExporter::new(sink_config.clone()) {
            Ok(mut exporter) => {
                let incident_refs: Vec<&Incident> = incidents.iter().collect();
                match exporter.export_fresh(&incident_refs, None) {
                    Ok(result) => {
                        eprintln!("[info] Exported {} incidents to {}", result.exported, result.output_path.display());
                        Some(result)
                    }
                    Err(e) => {
                        eprintln!("[error] Export failed: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("[error] Failed to create exporter: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Run ingest if requested
    let (ingest_stats, joined_count) = if let Some(ref path) = ingest_path {
        run_ingest(path)
    } else {
        (None, 0)
    };

    // Generate metrics report
    let metrics = IntegrationMetrics::new(output_dir.clone(), per_incident);
    let incident_refs: Vec<&Incident> = incidents.iter().collect();
    let facts: Vec<&Fact> = Vec::new(); // Would need to load from fact store

    let report = metrics.generate_report(
        &incident_refs,
        &facts,
        raw_event_count,
        export_result.as_ref(),
        ingest_stats.as_ref(),
        joined_count,
    );

    // Save report
    match metrics.save_report(&report) {
        Ok(path) => {
            eprintln!("[info] Metrics report saved to: {}", path.display());
        }
        Err(e) => {
            eprintln!("[error] Failed to save report: {}", e);
        }
    }

    // Print summary
    print_summary(&report);
}

fn print_help() {
    eprintln!(
        r#"
integration_metrics - Run metrics/proof harness over incidents and integrations

Usage:
  integration_metrics [OPTIONS]

Options:
  --incidents-dir DIR    Directory containing incident JSONL files
                         (default: /var/lib/edr/incidents)
  --segments-dir DIR     Directory containing segment files (for event count)
                         (default: /var/lib/edr/segments)
  --config PATH          Integration config file (YAML/JSON)
  --output DIR           Output directory for metrics
                         (default: /var/lib/edr/metrics)
  --per-incident         Include per-incident breakdown in report
  --export               Run export to configured sinks
  --ingest PATH          Run ingest from specified file
  --help                 Show this help

Example:
  integration_metrics \
    --incidents-dir /var/lib/edr/incidents \
    --config /etc/edr/integrations.yaml \
    --per-incident \
    --export

Output:
  - /var/lib/edr/metrics/run_<id>.json        Full metrics report
  - /var/lib/edr/metrics/run_<id>_summary.txt Human-readable summary
"#
    );
}

fn load_incidents(dir: &PathBuf) -> Vec<Incident> {
    let mut incidents = Vec::new();

    if !dir.exists() {
        eprintln!("[warn] Incidents directory does not exist: {}", dir.display());
        return incidents;
    }

    // Walk directory for JSONL files
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    for line in reader.lines().flatten() {
                        if let Ok(incident) = serde_json::from_str::<Incident>(&line) {
                            incidents.push(incident);
                        }
                    }
                }
            } else if path.is_dir() {
                // Recurse into namespace directories
                incidents.extend(load_incidents(&path));
            }
        }
    }

    incidents
}

fn count_raw_events(dir: &PathBuf) -> u64 {
    let mut count = 0u64;

    if !dir.exists() {
        return 0;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    count += reader.lines().count() as u64;
                }
            }
        }
    }

    count
}

fn run_ingest(path: &PathBuf) -> (Option<edr_locald::integrations::ingest::IngestStats>, u64) {
    use edr_locald::integrations::config::{IngestSourceConfig, IngestSourceType, JoinStrategy};

    let config = IngestSourceConfig {
        source_id: "cli_ingest".to_string(),
        source_type: IngestSourceType::JsonlFile,
        input_path: Some(path.clone()),
        listen_addr: None,
        vendor: "manual".to_string(),
        join_strategy: JoinStrategy::IpTimeWindow,
        join_window_seconds: 300,
        poll_interval_ms: 1000,
    };

    let mut ingester = VendorAlertIngester::new("cli_host");
    if let Err(e) = ingester.add_source_from_config(&config) {
        eprintln!("[error] Failed to add ingest source: {}", e);
        return (None, 0);
    }

    match ingester.poll() {
        Ok(facts) => {
            eprintln!("[info] Ingested {} facts from {}", facts.len(), path.display());
            let stats = ingester.stats();
            (Some(stats), 0) // joined_count would need actual join logic
        }
        Err(e) => {
            eprintln!("[error] Ingest failed: {}", e);
            (None, 0)
        }
    }
}

fn print_summary(report: &MetricsReport) {
    eprintln!("\n═══════════════════════════════════════════════════════════════");
    eprintln!("  METRICS SUMMARY");
    eprintln!("═══════════════════════════════════════════════════════════════");
    eprintln!("  Run ID:              {}", report.run_id);
    eprintln!("  Total Incidents:     {}", report.summary.total_incidents);
    eprintln!("  Average EQS:         {:.1}%", report.summary.avg_eqs * 100.0);
    eprintln!(
        "  EQS Range:           {:.1}% - {:.1}%",
        report.summary.min_eqs * 100.0,
        report.summary.max_eqs * 100.0
    );
    eprintln!(
        "  Events→Facts:        {:.1}:1",
        report.summary.events_to_facts_ratio
    );
    eprintln!(
        "  Facts→Incidents:     {:.1}:1",
        report.summary.facts_to_incidents_ratio
    );
    eprintln!(
        "  Duplicate Rate:      {:.2}%",
        report.summary.duplicate_rate * 100.0
    );
    eprintln!("───────────────────────────────────────────────────────────────");
    eprintln!("  Exported:            {}", report.export_metrics.incidents_exported);
    eprintln!("  Ingested Events:     {}", report.ingest_metrics.events_read);
    eprintln!("  Ingested Facts:      {}", report.ingest_metrics.facts_created);
    eprintln!(
        "  Join Rate:           {:.1}%",
        report.ingest_metrics.join_rate * 100.0
    );
    eprintln!("═══════════════════════════════════════════════════════════════\n");
}
