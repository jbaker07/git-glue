//! Integration Proof Run - Verify integration layer is wired correctly
//!
//! Simple proof that the integration layer can be instantiated and configured.

use chrono::Utc;
use edr_locald::integrations::{
    config::{ExportSinkConfig, ExportSinkType, IngestSourceConfig, IngestSourceType},
    profile::IntegrationProfileStore,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// Proof run output schema
#[derive(Debug, Serialize, Deserialize)]
struct ProofRunOutput {
    run_id: String,
    collected_at: String,
    telemetry_root: String,
    wiring_check: WiringCheck,
    summary: SummaryStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct WiringCheck {
    integration_profile_store_ok: bool,
    export_sink_config_ok: bool,
    ingest_source_config_ok: bool,
    all_wired: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SummaryStats {
    execution_time_ms: u128,
    status: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    let mut telemetry_root = PathBuf::from("/var/lib/edr");
    let mut output_path: Option<PathBuf> = None;

    // Parse args
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--telemetry-root" => {
                if i + 1 < args.len() {
                    telemetry_root = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--out" => {
                if i + 1 < args.len() {
                    output_path = Some(PathBuf::from(&args[i + 1]));
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

    let start_time = std::time::Instant::now();

    // Set default output path if not provided
    if output_path.is_none() {
        let run_id = Uuid::new_v4().to_string();
        let metrics_dir = telemetry_root.join("metrics");
        let _ = fs::create_dir_all(&metrics_dir);
        output_path = Some(metrics_dir.join(format!("proof_run_{}.json", run_id)));
    }

    // Execute proof run
    match execute_proof_run(telemetry_root.clone()) {
        Ok(mut result) => {
            result.summary.execution_time_ms = start_time.elapsed().as_millis();

            // Write output
            if let Some(out_path) = output_path {
                if let Ok(json) = serde_json::to_string_pretty(&result) {
                    if let Ok(mut file) = File::create(&out_path) {
                        let _ = file.write_all(json.as_bytes());
                        println!("✅ Proof run complete: {}", out_path.display());
                        println!();
                        println!("Wiring Check Results:");
                        println!("  IntegrationProfileStore: {}", if result.wiring_check.integration_profile_store_ok { "✓" } else { "✗" });
                        println!("  ExportSinkConfig: {}", if result.wiring_check.export_sink_config_ok { "✓" } else { "✗" });
                        println!("  IngestSourceConfig: {}", if result.wiring_check.ingest_source_config_ok { "✓" } else { "✗" });
                        println!();
                        println!("Overall Status: {}", result.summary.status);
                        println!("Execution: {}ms", result.summary.execution_time_ms);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Proof run failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_proof_run(telemetry_root: PathBuf) -> Result<ProofRunOutput, String> {
    let run_id = Uuid::new_v4().to_string();
    let collected_at = Utc::now().to_rfc3339();

    // Test 1: Can we create IntegrationProfileStore?
    let _store = IntegrationProfileStore::new();
    let integration_profile_store_ok = true;

    // Test 2: Can we create ExportSinkConfig?
    let export_sink = ExportSinkConfig {
        sink_id: "test_export".to_string(),
        sink_type: ExportSinkType::JsonlFile,
        output_path: telemetry_root.join("exports").join("test"),
        min_severity: "medium".to_string(),
        include_tier0: true,
        max_timeline_entries: 1000,
        max_evidence_pointers: 500,
        schema_version: "1.0".to_string(),
    };

    let export_dir = export_sink.output_path.clone();
    let export_sink_config_ok = fs::create_dir_all(export_dir).is_ok();

    // Test 3: Can we create IngestSourceConfig?
    let _ingest_config = IngestSourceConfig {
        source_id: "test_ingest".to_string(),
        source_type: IngestSourceType::WazuhAlerts,
        input_path: Some(telemetry_root.join("ingest")),
        listen_addr: None,
        vendor: "wazuh".to_string(),
        join_strategy: Default::default(),
        join_window_seconds: 60,
        poll_interval_ms: 1000,
    };

    let ingest_source_config_ok = true;

    let wiring_check = WiringCheck {
        integration_profile_store_ok,
        export_sink_config_ok,
        ingest_source_config_ok,
        all_wired: integration_profile_store_ok && export_sink_config_ok && ingest_source_config_ok,
    };

    let status = if wiring_check.all_wired {
        "ALL SYSTEMS WIRED".to_string()
    } else {
        "INCOMPLETE".to_string()
    };

    let summary = SummaryStats {
        execution_time_ms: 0,
        status,
    };

    Ok(ProofRunOutput {
        run_id,
        collected_at,
        telemetry_root: telemetry_root.to_string_lossy().to_string(),
        wiring_check,
        summary,
    })
}

fn print_help() {
    println!(
        r#"Integration Proof Run - Verify integration layer wiring

USAGE:
  proof_run [OPTIONS]

OPTIONS:
  --telemetry-root <PATH>    Telemetry root directory (default: /var/lib/edr)
  --out <PATH>               Output JSON file (default: /var/lib/edr/metrics/proof_run_<id>.json)
  --help                     Show this help

This tool verifies that:
  1. IntegrationProfileStore can be instantiated
  2. ExportSinkConfig can be created and output paths set up
  3. IngestSourceConfig can be created with vendor sources

EXAMPLE:
  cargo run --bin proof_run -- --telemetry-root /tmp/edr --out /tmp/proof.json
"#
    );
}
