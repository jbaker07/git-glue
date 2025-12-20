// windows/sensors/sysmon_adapter.rs
// Sysmon event adapter (auto-detect availability; degrade without error)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysmonEvent {
    pub event_id: u32,
    pub ts: DateTime<Utc>,
    pub computer: String,
    pub utc_time: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct SysmonAdapter {
    available: bool,
}

impl Default for SysmonAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SysmonAdapter {
    pub fn new() -> Self {
        // In real implementation, check for Sysmon log availability
        // For now, stub with availability flag
        Self { available: false }
    }

    /// Check if Sysmon is available on this system
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Convert Sysmon event to canonical core::Event
    /// Priority event IDs: 1 (Process), 3 (Network), 7 (ImageLoad), 22 (DNS), 23 (FileCreate)
    pub fn to_canonical_event(&self, event: &SysmonEvent) -> Option<crate::Event> {
        use crate::event::EventType;

        match event.event_id {
            1 => {
                // Process creation
                let exe = event.fields.get("Image").cloned();
                let argv = event.fields.get("CommandLine").cloned().map(|c| vec![c]);
                let pid = event.fields.get("ProcessId").and_then(|s| s.parse().ok());
                let ppid = event.fields.get("ParentProcessId").and_then(|s| s.parse().ok());
                let user = event.fields.get("User").cloned();

                return Some(crate::Event {
                    ts: event.ts,
                    event_type: EventType::Execve,
                    pid,
                    ppid,
                    exe,
                    argv,
                    tags: vec!["windows_sysmon".to_string(), "process".to_string(), user.unwrap_or_default()],
                    ..Default::default()
                });
            },
            3 => {
                // Network connection
                let exe = event.fields.get("Image").cloned();
                let pid = event.fields.get("ProcessId").and_then(|s| s.parse().ok());
                let dest_ip = event.fields.get("DestinationIp").cloned();
                let dest_port = event.fields.get("DestinationPort").and_then(|s| s.parse().ok());

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
                    exe,
                    sock: Some(sock),
                    tags: vec!["windows_sysmon".to_string(), "network".to_string()],
                    ..Default::default()
                });
            },
            7 => {
                // Image loaded
                let exe = event.fields.get("Image").cloned();
                let image_loaded = event.fields.get("ImageLoaded").cloned();
                let pid = event.fields.get("ProcessId").and_then(|s| s.parse().ok());

                return Some(crate::Event {
                    ts: event.ts,
                    event_type: EventType::Mmap,
                    pid,
                    exe,
                    tags: vec!["windows_sysmon".to_string(), "image_load".to_string(), image_loaded.unwrap_or_default()],
                    ..Default::default()
                });
            },
            22 => {
                // DNS query (Sysmon 22)
                return Some(crate::Event {
                    ts: event.ts,
                    event_type: EventType::Other,
                    tags: vec!["windows_sysmon".to_string(), "dns".to_string()],
                    ..Default::default()
                });
            },
            23 => {
                // File created (Sysmon 23)
                let file_name = event.fields.get("FileName").cloned();
                let exe = event.fields.get("Image").cloned();
                let pid = event.fields.get("ProcessId").and_then(|s| s.parse().ok());

                return Some(crate::Event {
                    ts: event.ts,
                    event_type: EventType::Other,
                    pid,
                    exe,
                    tags: vec!["windows_sysmon".to_string(), "file_create".to_string(), file_name.unwrap_or_default()],
                    ..Default::default()
                });
            },
            _ => {}
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sysmon_adapter_new() {
        let adapter = SysmonAdapter::new();
        assert!(!adapter.is_available());
    }
}
