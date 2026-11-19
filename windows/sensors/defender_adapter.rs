// windows/sensors/defender_adapter.rs
// Windows Defender event adaptation
// This is a stub for cross-platform compilation

#[derive(Debug)]
pub struct DefenderAdapter;

impl DefenderAdapter {
    pub fn adapt(_record: &super::evtx_collector::EvtxRecord) -> Option<crate::core::Event> {
        // TODO: Real implementation for Defender events
        None
    }
}
