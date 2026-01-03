//! cloud_tracker sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect cloud_tracker events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
