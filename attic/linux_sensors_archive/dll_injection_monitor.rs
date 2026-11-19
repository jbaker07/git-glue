//! dll_injection_monitor sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect dll_injection_monitor events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
