// windows/sensors/sysmon_adapter.rs
// Sysmon event adaptation - convert Sysmon events to canonical Event format
// This is a stub for cross-platform compilation

#[derive(Debug)]
pub struct SysmonAdapter;

impl SysmonAdapter {
    pub fn adapt(_record: &super::evtx_collector::EvtxRecord) -> Option<crate::core::Event> {
        // TODO: Real implementation converts Sysmon events to canonical format
        None
    }
}
