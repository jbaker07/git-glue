// windows/sensors/sysmon_adapter_ext.rs
// Extended Sysmon event adaptation
// This is a stub for cross-platform compilation

#[derive(Debug)]
pub struct SysmonAdapterExt;

impl SysmonAdapterExt {
    pub fn adapt(_record: &super::evtx_collector::EvtxRecord) -> Option<crate::core::Event> {
        // TODO: Real implementation for extended Sysmon events
        None
    }
}
