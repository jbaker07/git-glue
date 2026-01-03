//! password_spray sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect password_spray events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
