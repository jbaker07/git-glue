// windows/sensors/evtx_collector.rs
// EVTX event log file reading
// Real implementation uses Windows API (EvtOpenLog, EvtQuery, etc.)
// This is a stub for cross-platform compilation

#[derive(Debug, Clone)]
pub struct EvtxRecord {
    pub event_id: u32,
    pub level: String,
    pub provider: String,
    pub message: String,
}

#[derive(Debug)]
pub struct EvtxCollector;

impl EvtxCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn enabled_logs(&self) -> Vec<&'static str> {
        #[cfg(target_os = "windows")]
        {
            vec!["Sysmon", "Security", "System"]
        }
        #[cfg(not(target_os = "windows"))]
        {
            vec![] // No event logs on non-Windows
        }
    }

    pub fn poll(&self, log: &str) -> anyhow::Result<Vec<EvtxRecord>> {
        // TODO: Real implementation uses EvtOpenLog, EvtQuery, EvtNext
        Ok(vec![])
    }
}
