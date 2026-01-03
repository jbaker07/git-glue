//! replay_writer sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect replay_writer events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
