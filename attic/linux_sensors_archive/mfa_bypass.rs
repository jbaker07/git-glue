//! mfa_bypass sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect mfa_bypass events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
