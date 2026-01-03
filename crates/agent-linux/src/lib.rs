//! Linux EDR Agent
//!
//! Captures process, network, and file events from Linux systems
//! using eBPF and procfs.

pub mod telemetry;
pub mod telemetry_types;

pub mod capture_linux_rotating;
pub mod event_builders;
pub mod host;
pub mod sensors;
pub mod sensors_v2;

#[cfg(target_os = "linux")]
pub mod ebpf;

pub use telemetry::TelemetryRecord;
pub use telemetry_types::TelemetryOutput;
