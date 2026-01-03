//! geo_ip_anomaly sensor
#![allow(dead_code)]

use crate::core::Event;
use crate::linux::host::HostCtx;

/// Public contract: collect geo_ip_anomaly events  
pub fn collect(_host: &HostCtx) -> Vec<Event> {
    vec![]
}
