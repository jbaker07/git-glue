//! mem_scan sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect mem_scan events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
