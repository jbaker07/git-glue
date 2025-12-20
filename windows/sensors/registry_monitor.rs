// windows/sensors/registry_monitor.rs
// Registry modification monitoring (Run keys, services, startup, privilege settings)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEvent {
    pub ts: DateTime<Utc>,
    pub hive: String,           // HKLM, HKCU, HKU, HKCR
    pub key_path: String,
    pub value_name: String,
    pub value_type: String,     // REG_SZ, REG_DWORD, etc.
    pub old_data: Option<String>,
    pub new_data: Option<String>,
    pub pid: Option<i32>,
    pub exe: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug)]
pub struct RegistryMonitor {
    monitored_keys: Vec<String>,
}

impl Default for RegistryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryMonitor {
    pub fn new() -> Self {
        let monitored_keys = vec![
            "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
            "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce".to_string(),
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce".to_string(),
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\RunMRU".to_string(),
            "HKLM\\Software\\Classes\\.exe".to_string(),
            "HKLM\\System\\CurrentControlSet\\Services".to_string(),
            "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Image File Execution Options".to_string(),
            "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System".to_string(),
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System".to_string(),
            "HKLM\\Software\\Policies\\Microsoft\\Windows Defender".to_string(),
        ];

        Self { monitored_keys }
    }

    pub fn monitored_keys(&self) -> &[String] {
        &self.monitored_keys
    }

    pub fn is_suspicious_registry_change(&self, event: &RegistryEvent) -> bool {
        // Check for suspicious patterns
        if event.key_path.contains("Run") && 
           event.new_data.as_ref().map_or(false, |d| {
               d.contains("\\Temp\\") || 
               d.contains("\\AppData\\Local\\Temp") ||
               d.contains("Downloads") ||
               d.contains("AppData\\Roaming\\Microsoft\\Windows\\Start Menu")
           }) {
            return true;
        }

        // Services with suspicious paths
        if event.key_path.contains("Services") &&
           event.new_data.as_ref().map_or(false, |d| {
               d.contains("\\Temp\\") ||
               d.contains("cmd.exe") ||
               d.contains("powershell.exe")
           }) {
            return true;
        }

        // Defender bypass attempts
        if event.key_path.contains("Windows Defender") &&
           event.value_name.contains("Disable") &&
           event.new_data.as_ref().map_or(false, |d| d == "1") {
            return true;
        }

        false
    }

    pub fn to_canonical_event(&self, event: &RegistryEvent) -> Option<crate::Event> {
        if !self.is_suspicious_registry_change(event) {
            return None;
        }

        let exe = event.exe.clone();
        let pid = event.pid;

        Some(crate::Event {
            ts: event.ts,
            event_type: crate::event::EventType::Other,
            pid,
            exe,
            tags: vec![
                "windows_registry".to_string(),
                "registry_modification".to_string(),
                event.hive.clone(),
                event.key_path.clone(),
            ],
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitored_keys() {
        let monitor = RegistryMonitor::new();
        assert!(monitor.monitored_keys().len() > 0);
    }

    #[test]
    fn test_suspicious_run_key() {
        let monitor = RegistryMonitor::new();
        let event = RegistryEvent {
            ts: Utc::now(),
            hive: "HKCU".to_string(),
            key_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
            value_name: "Malware".to_string(),
            value_type: "REG_SZ".to_string(),
            old_data: None,
            new_data: Some("C:\\Users\\Public\\AppData\\Local\\Temp\\evil.exe".to_string()),
            pid: Some(1234),
            exe: Some("regedit.exe".to_string()),
            user: Some("admin".to_string()),
        };
        assert!(monitor.is_suspicious_registry_change(&event));
    }
}
