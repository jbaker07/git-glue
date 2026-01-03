//! job_sched_monitor sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect job_sched_monitor events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
