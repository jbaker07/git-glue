//! entropy_exec_monitor sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect entropy_exec_monitor events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
