//! file_tamper_monitor sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect file_tamper_monitor events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
