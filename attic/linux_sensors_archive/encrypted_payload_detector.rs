//! encrypted_payload_detector sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect encrypted_payload_detector events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
