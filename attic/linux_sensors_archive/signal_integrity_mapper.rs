//! signal_integrity_mapper sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect signal_integrity_mapper events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
