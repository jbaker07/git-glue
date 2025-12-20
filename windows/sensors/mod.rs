// windows/sensors/mod.rs
// Windows telemetry collectors + adapters → canonical core::Event
// Supports: Event Logs, Sysmon, ETW (feature-gated), Defender/AMSI

pub mod evtx_collector;
pub mod sysmon_adapter;
pub mod etw_adapter;
pub mod defender_adapter;
pub mod capture_windows_rotating;

pub use evtx_collector::EvtxCollector;
pub use sysmon_adapter::SysmonAdapter;
pub use etw_adapter::EtwAdapter;
pub use defender_adapter::DefenderAdapter;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WindowsSensorKind {
    #[serde(rename = "windows_evtx")]
    EvtxLog,
    #[serde(rename = "windows_sysmon")]
    Sysmon,
    #[serde(rename = "windows_etw")]
    Etw,
    #[serde(rename = "windows_defender")]
    Defender,
    #[serde(rename = "windows_amsi")]
    Amsi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsCapabilityConfig {
    /// Enable Sysmon adapters if available
    pub enable_sysmon: bool,
    /// Enable ETW collection (requires elevation; feature-gated)
    pub enable_etw: bool,
    /// Enable Defender/AMSI logs if present
    pub enable_defender: bool,
    /// Rate limit for high-volume sources (events/sec)
    pub rate_limit: u32,
    /// Redact sensitive fields (PowerShell commands, etc.)
    pub redact_sensitive: bool,
}

impl Default for WindowsCapabilityConfig {
    fn default() -> Self {
        Self {
            enable_sysmon: true,
            enable_etw: cfg!(feature = "windows_etw"),
            enable_defender: true,
            rate_limit: 1000,
            redact_sensitive: true,
        }
    }
}
