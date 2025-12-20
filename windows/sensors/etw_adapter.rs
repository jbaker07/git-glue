// windows/sensors/etw_adapter.rs
// ETW (Event Tracing for Windows) adapter — best-effort, feature-gated
// Covers: Process/Thread, ImageLoad, TCP connect, DNS (if feasible)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtwEvent {
    pub provider: String,
    pub event_name: String,
    pub ts: DateTime<Utc>,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct EtwAdapter {
    available: bool,
}

impl Default for EtwAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl EtwAdapter {
    pub fn new() -> Self {
        // ETW only available on Windows with elevation
        #[cfg(target_os = "windows")]
        let available = true;
        #[cfg(not(target_os = "windows"))]
        let available = false;

        Self { available }
    }

    pub fn is_available(&self) -> bool {
        self.available && cfg!(feature = "windows_etw")
    }

    /// Convert ETW event to canonical core::Event
    pub fn to_canonical_event(&self, event: &EtwEvent) -> Option<crate::Event> {
        use crate::event::EventType;

        // Kernel Process/Thread provider
        if event.provider == "Microsoft-Windows-Kernel-Process" {
            match event.event_name.as_str() {
                "ProcessStart" => {
                    let exe = event.fields.get("ImageName").cloned();
                    let pid = event.fields.get("ProcessID").and_then(|s| s.parse().ok());
                    let ppid = event.fields.get("ParentID").and_then(|s| s.parse().ok());

                    return Some(crate::Event {
                        ts: event.ts,
                        event_type: EventType::Execve,
                        pid,
                        ppid,
                        exe,
                        tags: vec!["windows_etw".to_string(), "process".to_string()],
                        ..Default::default()
                    });
                },
                "ProcessStop" => {
                    let pid = event.fields.get("ProcessID").and_then(|s| s.parse().ok());
                    return Some(crate::Event {
                        ts: event.ts,
                        event_type: EventType::Other,
                        pid,
                        tags: vec!["windows_etw".to_string(), "process_end".to_string()],
                        ..Default::default()
                    });
                },
                _ => {}
            }
        }

        // ImageLoad provider
        if event.provider == "Microsoft-Windows-Kernel-ImageLoad" {
            let exe = event.fields.get("ImageName").cloned();
            let pid = event.fields.get("ProcessID").and_then(|s| s.parse().ok());

            return Some(crate::Event {
                ts: event.ts,
                event_type: EventType::Mmap,
                pid,
                exe,
                tags: vec!["windows_etw".to_string(), "image_load".to_string()],
                ..Default::default()
            });
        }

        // TCP/IP provider (network connect)
        if event.provider == "Microsoft-Windows-TCPIP" {
            let dest_ip = event.fields.get("daddr").cloned();
            let dest_port = event.fields.get("dport").and_then(|s| s.parse().ok());
            let pid = event.fields.get("ProcessID").and_then(|s| s.parse().ok());

            let sock = crate::event::SockMeta {
                family: Some(2), // AF_INET
                raddr: dest_ip,
                rport: dest_port,
                ..Default::default()
            };

            return Some(crate::Event {
                ts: event.ts,
                event_type: EventType::Other,
                pid,
                sock: Some(sock),
                tags: vec!["windows_etw".to_string(), "network".to_string()],
                ..Default::default()
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etw_adapter_new() {
        let adapter = EtwAdapter::new();
        // Should be available only on Windows with feature flag
        assert_eq!(adapter.is_available(), cfg!(all(target_os = "windows", feature = "windows_etw")));
    }
}
