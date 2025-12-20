// windows/sensors/evtx_collector.rs
// Windows Event Log collector (Security, PowerShell, WMI, TaskScheduler, System, Application)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvtxRecord {
    pub log_name: String,
    pub event_id: u32,
    pub ts: DateTime<Utc>,
    pub computer: String,
    pub level: String,
    pub provider: String,
    pub fields: BTreeMap<String, String>,
    pub raw_xml: Option<String>,
}

#[derive(Debug)]
pub struct EvtxCollector {
    enabled_logs: Vec<String>,
    event_id_filters: BTreeMap<String, Vec<u32>>,
}

impl Default for EvtxCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl EvtxCollector {
    pub fn new() -> Self {
        let mut event_id_filters = BTreeMap::new();
        
        // Security log event IDs of interest
        event_id_filters.insert("Security".to_string(), vec![
            4688,  // Process creation
            4689,  // Process terminated
            4720,  // User account created
            4722,  // User account enabled
            4725,  // User account disabled
            4740,  // User account locked
            4726,  // User account deleted
            4728,  // Member added to security-enabled group
            4732,  // Member added to security-enabled local group
            4756,  // Member added to security-enabled universal group
            4767,  // User account unlocked
            4768,  // Kerberos auth ticket requested
            4769,  // Kerberos service ticket requested
            4770,  // Kerberos service ticket renewed
            4771,  // Kerberos pre-auth failed
            4776,  // Auth attempt (NTLM)
            4798,  // Group enumeration
            4799,  // Security group enumeration
            4964,  // Special groups assigned to new logon
            5140,  // Network share accessed
            5145,  // Network share checked
        ]);

        // PowerShell Operational
        event_id_filters.insert("Microsoft-Windows-PowerShell/Operational".to_string(), vec![
            4100,  // Pipeline execution details
            4103,  // Module logging
            4104,  // Script block logging
            4105,  // Command start/stop
        ]);

        // WMI Activity
        event_id_filters.insert("Microsoft-Windows-WMI-Activity/Operational".to_string(), vec![
            5857,  // WMI event consumer created/modified
            5858,  // WMI event consumer deleted
            5859,  // WMI event filter created/modified
            5860,  // WMI event filter deleted
            5861,  // WMI event consumer binding
        ]);

        // Task Scheduler
        event_id_filters.insert("Microsoft-Windows-TaskScheduler/Operational".to_string(), vec![
            106,  // Task registered
            140,  // Task updated
            141,  // Task deleted
            200,  // Task executed
        ]);

        Self {
            enabled_logs: vec![
                "Security".to_string(),
                "System".to_string(),
                "Application".to_string(),
                "Microsoft-Windows-PowerShell/Operational".to_string(),
                "Microsoft-Windows-WMI-Activity/Operational".to_string(),
                "Microsoft-Windows-TaskScheduler/Operational".to_string(),
            ],
            event_id_filters,
        }
    }

    /// Returns list of enabled Windows Event Log names
    pub fn enabled_logs(&self) -> &[String] {
        &self.enabled_logs
    }

    /// Check if an event ID should be collected from this log
    pub fn should_collect(&self, log_name: &str, event_id: u32) -> bool {
        match self.event_id_filters.get(log_name) {
            Some(ids) => ids.contains(&event_id),
            None => true, // Collect all from unlisted logs
        }
    }

    /// Convert EVTX record to canonical core::Event
    pub fn to_canonical_event(&self, record: &EvtxRecord) -> Option<crate::Event> {
        use crate::event::EventType;
        
        // Example: Security event 4688 = Process creation
        if record.log_name == "Security" {
            match record.event_id {
                4688 => {
                    // Process creation: extract command line, image, parent image
                    let exe = record.fields.get("NewProcessName").cloned();
                    let argv = record.fields.get("CommandLine").cloned().map(|c| vec![c]);
                    let pid = record.fields.get("NewProcessId").and_then(|s| s.parse().ok());
                    let ppid = record.fields.get("ParentProcessId").and_then(|s| s.parse().ok());
                    let uid = record.fields.get("SubjectUserSid").cloned().map(|_| 0);

                    return Some(crate::Event {
                        ts: record.ts,
                        event_type: EventType::Execve,
                        pid,
                        ppid,
                        uid,
                        exe,
                        argv,
                        tags: vec!["windows_evtx".to_string(), "process".to_string()],
                        ..Default::default()
                    });
                },
                4776 | 4768 => {
                    // Authentication attempt
                    return Some(crate::Event {
                        ts: record.ts,
                        event_type: EventType::Other,
                        tags: vec!["windows_evtx".to_string(), "auth".to_string()],
                        ..Default::default()
                    });
                },
                5140 => {
                    // Network share accessed
                    return Some(crate::Event {
                        ts: record.ts,
                        event_type: EventType::Other,
                        tags: vec!["windows_evtx".to_string(), "network".to_string()],
                        ..Default::default()
                    });
                },
                _ => {}
            }
        }

        if record.log_name == "Microsoft-Windows-PowerShell/Operational" {
            match record.event_id {
                4104 => {
                    // Script block executed
                    let cmd = record.fields.get("ScriptBlockText").cloned();
                    return Some(crate::Event {
                        ts: record.ts,
                        event_type: EventType::Other,
                        argv: cmd.map(|c| vec![c]),
                        tags: vec!["windows_powershell".to_string(), "script_exec".to_string()],
                        ..Default::default()
                    });
                },
                _ => {}
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_filters() {
        let collector = EvtxCollector::new();
        assert!(collector.should_collect("Security", 4688));
        assert!(!collector.should_collect("Security", 9999));
    }
}
