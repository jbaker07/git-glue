//! auth_pipe_listener sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect auth_pipe_listener events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
