//! persistence_watch sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect persistence_watch events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
