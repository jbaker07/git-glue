//! Linux sensor abstraction layer
//! All sensors implement the LinuxSensor trait and emit pure core::Event
//! No side effects (no telemetry, no gnn hooks, no trust scoring)
//! capture_linux_rotating.rs is the sole writer and EvidencePtr authority

use anyhow::Result;
use crate::core::Event;

/// Common interface for Linux sensors
pub trait LinuxSensor: Send {
    /// Human-readable sensor name
    fn name(&self) -> &'static str;
    
    /// Poll the sensor and return events
    /// Sensors must emit pure Events without side effects
    fn poll(&mut self, host: &str) -> Result<Vec<Event>>;
}

// Helper for sensors being refactored
mod stub_impl;
pub use stub_impl::stub_poll;

/// Tier-0 sensors: procfs-based, always available
pub mod procfs_process;

/// Tier-1+ sensors: eBPF, audit, file monitoring (to be refactored)
pub mod alert_engine;
pub mod auth_monitor;
pub mod auth_pipe_listener;
pub mod cloud_tracker;
pub mod container_monitor;
pub mod dll_injection_monitor;
pub mod encrypted_payload_detector;
pub mod entropy_exec_monitor;
pub mod file_hash_watcher;
pub mod file_tamper_monitor;
pub mod geo_ip_anomaly;
pub mod job_sched_monitor;
pub mod lkm_monitor;
pub mod logon_tracker;
pub mod mem_scan;
pub mod mfa_bypass;
pub mod net_watch;
pub mod password_spray;
pub mod persistence_watch;
pub mod privilege_monitor;
pub mod process_injection;
pub mod process_monitor;
pub mod replay_writer;
pub mod script_monitor;
pub mod signal_integrity_mapper;
pub mod suspicious_ipc;
pub mod telemetry_fingerprint;
pub mod trust_state_restorer;
pub mod usb_monitor;
pub mod user_tracker;

// Minimal trait implementations for sensors being refactored
// Each sensor module will be enhanced incrementally to emit core::Event

pub struct ProcfsProcessSensorImpl;
impl LinuxSensor for ProcfsProcessSensorImpl {
    fn name(&self) -> &'static str {
        "procfs_process"
    }
    fn poll(&mut self, host: &str) -> Result<Vec<Event>> {
        procfs_process::poll_processes(host)
    }
}

/// Factory to create sensor instances
pub fn create_sensors() -> Vec<Box<dyn LinuxSensor>> {
    vec![
        Box::new(ProcfsProcessSensorImpl),
        // Additional sensors can be added here as they're refactored
    ]
}
