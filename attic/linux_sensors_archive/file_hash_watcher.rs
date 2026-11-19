//! file_hash_watcher sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect file_hash_watcher events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
