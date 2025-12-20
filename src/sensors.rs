/// Sensor abstraction layer for macOS telemetry
/// 
/// Decouples event source from processing pipeline.
/// Supports future: OpenBSM → EndpointSecurity migration without touching locald/ui/schemas

use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// NormalizedEvent is the canonical in-process event format (identical to existing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedEvent {
    pub schema_version: u32,
    pub ts_ms: u64,
    pub host: String,
    pub event_type: String,
    pub subject: SubjectInfo,
    pub process: ProcessInfo,
    pub payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_bsm: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectInfo {
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub auid: Option<u32>,
    pub euid: Option<u32>,
    pub egid: Option<u32>,
    pub session_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: Option<u32>,
    pub ppid: Option<u32>,
    pub exe: Option<String>,
    pub argv: Option<Vec<String>>,
    pub cwd: Option<String>,
}

/// Stop token for graceful shutdown
pub type StopToken = Arc<AtomicBool>;

/// EventSink receives normalized events from sensor
pub trait EventSink {
    fn on_event(&mut self, evt: NormalizedEvent) -> Result<(), String>;
}

/// Sensor produces NormalizedEvents
pub trait Sensor: Send {
    fn name(&self) -> &'static str;
    fn run(&mut self, sink: &mut dyn EventSink, stop: StopToken) -> Result<(), String>;
}

/// OpenBSM sensor - reads /dev/auditpipe, parses BSM, produces NormalizedEvents
pub struct OpenBsmSensor {
    pub host: String,
}

impl OpenBsmSensor {
    pub fn new(host: String) -> Self {
        Self { host }
    }
}

impl Sensor for OpenBsmSensor {
    fn name(&self) -> &'static str {
        "openbsm"
    }

    fn run(&mut self, _sink: &mut dyn EventSink, _stop: StopToken) -> Result<(), String> {
        // This is a stub; actual implementation is in capture_macos_rotating.rs
        // which will import and use this trait.
        Err("OpenBsmSensor.run() must be implemented in binary".to_string())
    }
}
