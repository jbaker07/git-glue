// windows/sensors/wmi_monitor.rs
// WMI (Windows Management Instrumentation) execution and subscription monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WmiEvent {
    pub ts: DateTime<Utc>,
    pub event_type: WmiEventType,
    pub command: Option<String>,
    pub class: Option<String>,
    pub namespace: Option<String>,
    pub pid: Option<i32>,
    pub exe: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WmiEventType {
    ProcessCreated,
    EventSubscription,
    EventConsumer,
    EventFilter,
    ScriptExecution,
    ClassModified,
}

#[derive(Debug)]
pub struct WmiMonitor;

impl WmiMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn is_suspicious_wmi(&self, event: &WmiEvent) -> bool {
        match event.event_type {
            WmiEventType::ProcessCreated => {
                // WMI process creation is inherently suspicious
                true
            },
            
            WmiEventType::EventSubscription | WmiEventType::EventConsumer | WmiEventType::EventFilter => {
                // WMI event subscriptions can be used for persistence
                if let Some(cmd) = &event.command {
                    cmd.contains("cmd.exe") ||
                    cmd.contains("powershell.exe") ||
                    cmd.contains("cscript.exe") ||
                    cmd.contains("wscript.exe")
                } else {
                    true // Any subscription is suspicious
                }
            },
            
            WmiEventType::ScriptExecution => {
                // WMI script execution is suspicious
                if let Some(cmd) = &event.command {
                    cmd.contains("-nop") ||
                    cmd.contains("-enc") ||
                    cmd.contains("-ep bypass") ||
                    cmd.contains("IEX") ||
                    cmd.contains("DownloadString")
                } else {
                    true
                }
            },
            
            WmiEventType::ClassModified => {
                // Modification of WMI classes is suspicious
                true
            },
        }
    }

    pub fn to_canonical_event(&self, event: &WmiEvent) -> Option<crate::Event> {
        if !self.is_suspicious_wmi(event) {
            return None;
        }

        Some(crate::Event {
            ts: event.ts,
            event_type: crate::event::EventType::Other,
            pid: event.pid,
            exe: event.exe.clone(),
            argv: event.command.as_ref().map(|c| vec![c.clone()]),
            tags: vec![
                "windows_wmi".to_string(),
                format!("{:?}", event.event_type).to_lowercase(),
                event.namespace.clone().unwrap_or_default(),
                "wmi_execution".to_string(),
            ],
            ..Default::default()
        })
    }
}

impl Default for WmiMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wmi_process_creation() {
        let monitor = WmiMonitor::new();
        let event = WmiEvent {
            ts: Utc::now(),
            event_type: WmiEventType::ProcessCreated,
            command: None,
            class: Some("Win32_Process".to_string()),
            namespace: Some("root\\cimv2".to_string()),
            pid: Some(1234),
            exe: Some("WmiPrvSE.exe".to_string()),
            user: Some("SYSTEM".to_string()),
        };
        assert!(monitor.is_suspicious_wmi(&event));
    }

    #[test]
    fn test_wmi_subscription_with_powershell() {
        let monitor = WmiMonitor::new();
        let event = WmiEvent {
            ts: Utc::now(),
            event_type: WmiEventType::EventSubscription,
            command: Some("powershell.exe -nop -enc AAAA==".to_string()),
            class: None,
            namespace: None,
            pid: None,
            exe: None,
            user: None,
        };
        assert!(monitor.is_suspicious_wmi(&event));
    }
}
