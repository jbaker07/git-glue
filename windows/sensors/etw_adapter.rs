// windows/sensors/etw_adapter.rs
// ETW event tracing adaptation - for WMI, WinRM, etc.
// This is a stub for cross-platform compilation

#[derive(Debug)]
pub struct EtwAdapter;

impl EtwAdapter {
    pub fn adapt(_record: &super::evtx_collector::EvtxRecord) -> Option<crate::core::Event> {
        // TODO: Real implementation for ETW events
        None
    }
}
