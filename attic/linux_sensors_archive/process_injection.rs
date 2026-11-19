//! process_injection sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect process_injection events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
