// windows/sensors/driver_monitor.rs
// Driver loading and kernel module execution monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverEvent {
    pub ts: DateTime<Utc>,
    pub event_type: DriverEventType,
    pub driver_name: String,
    pub driver_path: Option<String>,
    pub service_name: Option<String>,
    pub signed: Option<bool>,
    pub signature_issuer: Option<String>,
    pub pid: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverEventType {
    DriverLoaded,
    DriverUnloaded,
    ServiceCreated,
    DriverExecuted,
}

#[derive(Debug)]
pub struct DriverMonitor;

impl DriverMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn is_suspicious_driver(&self, event: &DriverEvent) -> bool {
        // Unsigned drivers are suspicious
        if let Some(false) = event.signed {
            return true;
        }

        // Drivers in suspicious locations
        if let Some(path) = &event.driver_path {
            let lower = path.to_lowercase();
            if lower.contains("\\temp\\") ||
               lower.contains("\\appdata\\") ||
               lower.contains("\\users\\") ||
               lower.contains("\\downloads\\") {
                return true;
            }
        }

        // Check for suspicious driver names (common in rootkits)
        let lower_name = event.driver_name.to_lowercase();
        if lower_name.contains("rootkit") ||
           lower_name.contains("hidden") ||
           lower_name.contains("stealth") ||
           lower_name.contains("kernel") {
            return true;
        }

        false
    }

    pub fn to_canonical_event(&self, event: &DriverEvent) -> Option<crate::Event> {
        if !self.is_suspicious_driver(event) {
            return None;
        }

        Some(crate::Event {
            ts: event.ts,
            event_type: crate::event::EventType::Other,
            pid: event.pid,
            tags: vec![
                "windows_driver".to_string(),
                format!("{:?}", event.event_type).to_lowercase(),
                event.driver_name.clone(),
                format!("signed_{}", event.signed.unwrap_or(false)),
            ],
            ..Default::default()
        })
    }
}

impl Default for DriverMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsigned_driver() {
        let monitor = DriverMonitor::new();
        let event = DriverEvent {
            ts: Utc::now(),
            event_type: DriverEventType::DriverLoaded,
            driver_name: "evil.sys".to_string(),
            driver_path: Some("C:\\Windows\\System32\\evil.sys".to_string()),
            service_name: Some("EvilService".to_string()),
            signed: Some(false),
            signature_issuer: None,
            pid: None,
        };
        assert!(monitor.is_suspicious_driver(&event));
    }

    #[test]
    fn test_driver_in_temp() {
        let monitor = DriverMonitor::new();
        let event = DriverEvent {
            ts: Utc::now(),
            event_type: DriverEventType::DriverLoaded,
            driver_name: "malware.sys".to_string(),
            driver_path: Some("C:\\Users\\Admin\\AppData\\Local\\Temp\\malware.sys".to_string()),
            service_name: None,
            signed: Some(true),
            signature_issuer: None,
            pid: None,
        };
        assert!(monitor.is_suspicious_driver(&event));
    }
}
