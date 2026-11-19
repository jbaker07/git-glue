//! telemetry_fingerprint sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect telemetry_fingerprint events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
