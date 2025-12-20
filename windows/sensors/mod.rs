// windows/sensors/mod.rs
// Windows telemetry collectors + adapters → canonical core::Event
// Supports: Event Logs, Sysmon, ETW (feature-gated), Defender/AMSI, Registry, File, Network, Process, WMI, Driver, Auth

pub mod evtx_collector;
pub mod sysmon_adapter;
pub mod etw_adapter;
pub mod defender_adapter;
pub mod registry_monitor;
pub mod file_monitor;
pub mod network_monitor;
pub mod process_monitor;
pub mod wmi_monitor;
pub mod driver_monitor;
pub mod auth_monitor;
pub mod capture_windows_rotating;

pub use evtx_collector::EvtxCollector;
pub use sysmon_adapter::SysmonAdapter;
pub use etw_adapter::EtwAdapter;
pub use defender_adapter::DefenderAdapter;
pub use registry_monitor::RegistryMonitor;
pub use file_monitor::FileMonitor;
pub use network_monitor::NetworkMonitor;
pub use process_monitor::ProcessMonitor;
pub use wmi_monitor::WmiMonitor;
pub use driver_monitor::DriverMonitor;
pub use auth_monitor::AuthMonitor;

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
    #[serde(rename = "windows_registry")]
    Registry,
    #[serde(rename = "windows_file")]
    File,
    #[serde(rename = "windows_network")]
    Network,
    #[serde(rename = "windows_process")]
    Process,
    #[serde(rename = "windows_wmi")]
    Wmi,
    #[serde(rename = "windows_driver")]
    Driver,
    #[serde(rename = "windows_auth")]
    Auth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsCapabilityConfig {
    /// Enable Sysmon adapters if available
    pub enable_sysmon: bool,
    /// Enable ETW collection (requires elevation; feature-gated)
    pub enable_etw: bool,
    /// Enable Defender/AMSI logs if present
    pub enable_defender: bool,
    /// Enable registry monitoring
    pub enable_registry: bool,
    /// Enable file system monitoring
    pub enable_file_monitor: bool,
    /// Enable network monitoring
    pub enable_network: bool,
    /// Enable process behavior monitoring
    pub enable_process: bool,
    /// Enable WMI monitoring
    pub enable_wmi: bool,
    /// Enable driver/kernel monitoring
    pub enable_driver: bool,
    /// Enable authentication monitoring
    pub enable_auth: bool,
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
            enable_registry: true,
            enable_file_monitor: true,
            enable_network: true,
            enable_process: true,
            enable_wmi: true,
            enable_driver: true,
            enable_auth: true,
            rate_limit: 1000,
            redact_sensitive: true,
        }
    }
}

