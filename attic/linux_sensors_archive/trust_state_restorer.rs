//! trust_state_restorer sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect trust_state_restorer events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
