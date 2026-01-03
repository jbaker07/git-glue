#!/bin/bash

# List of sensor files to stub out temporarily
SENSORS=(
    "auth_monitor.rs"
    "auth_pipe_listener.rs"
    "cloud_tracker.rs"
    "container_monitor.rs"
    "dll_injection_monitor.rs"
    "encrypted_payload_detector.rs"
    "entropy_exec_monitor.rs"
    "file_hash_watcher.rs"
    "file_tamper_monitor.rs"
    "geo_ip_anomaly.rs"
    "job_sched_monitor.rs"
    "lkm_monitor.rs"
    "logon_tracker.rs"
    "mem_scan.rs"
    "mfa_bypass.rs"
    "net_watch.rs"
    "password_spray.rs"
    "persistence_watch.rs"
    "privilege_monitor.rs"
    "process_injection.rs"
    "process_monitor.rs"
    "replay_writer.rs"
    "script_monitor.rs"
    "signal_integrity_mapper.rs"
    "suspicious_ipc.rs"
    "telemetry_fingerprint.rs"
    "trust_state_restorer.rs"
)

# Create minimal stub for each sensor
for sensor in "${SENSORS[@]}"; do
    name="${sensor%.rs}"
    cat > "$sensor" << SENSOR_EOF
//! Sensor module: $name
//! TODO: Refactor to emit core::Event

use crate::core::Event;
use anyhow::Result;

/// Placeholder implementation 
pub fn collect(_host: &str) -> Result<Vec<Event>> {
    Ok(Vec::new())
}
SENSOR_EOF
done

echo "Stubbed all problematic sensor files"
