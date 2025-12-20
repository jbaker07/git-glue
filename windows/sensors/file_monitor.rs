// windows/sensors/file_monitor.rs
// File system monitoring (creation, modification, deletion, suspicious patterns)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub ts: DateTime<Utc>,
    pub file_path: String,
    pub file_size: Option<u64>,
    pub file_hash: Option<String>,
    pub action: FileAction,
    pub pid: Option<i32>,
    pub exe: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileAction {
    Created,
    Modified,
    Deleted,
    Renamed,
    PermissionChanged,
}

#[derive(Debug)]
pub struct FileMonitor {
    suspicious_extensions: Vec<&'static str>,
    monitored_dirs: Vec<&'static str>,
}

impl Default for FileMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FileMonitor {
    pub fn new() -> Self {
        Self {
            suspicious_extensions: vec![
                ".exe", ".dll", ".scr", ".com", ".bat", ".cmd", ".vbs", ".js",
                ".ps1", ".vbe", ".jse", ".wsf", ".wsh", ".msi", ".zip", ".rar",
            ],
            monitored_dirs: vec![
                "\\Temp\\",
                "\\AppData\\Local\\Temp",
                "\\AppData\\Local\\Low",
                "\\Users\\Public\\",
                "\\Windows\\System32\\spool\\",
                "\\Recycle.Bin\\",
                "\\PerfLogs\\",
            ],
        }
    }

    pub fn is_suspicious_file(&self, event: &FileEvent) -> bool {
        let path = &event.file_path;

        // Check for suspicious locations
        for dir in &self.monitored_dirs {
            if path.contains(dir) {
                // Suspicious extension in sensitive location
                for ext in &self.suspicious_extensions {
                    if path.to_lowercase().ends_with(ext) {
                        return true;
                    }
                }
            }
        }

        // Double extension (.txt.exe, .pdf.exe)
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() > 2 {
            if let Some(last) = parts.last() {
                for ext in &self.suspicious_extensions {
                    if last.to_lowercase() == ext.trim_start_matches('.') {
                        return true;
                    }
                }
            }
        }

        // Executable in Desktop/Downloads
        if (path.contains("\\Desktop\\") || path.contains("\\Downloads\\")) &&
           path.to_lowercase().ends_with(".exe") {
            return true;
        }

        false
    }

    pub fn to_canonical_event(&self, event: &FileEvent) -> Option<crate::Event> {
        if !self.is_suspicious_file(event) && event.action == FileAction::Created {
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
                "windows_file".to_string(),
                format!("file_{:?}", event.action).to_lowercase(),
                event.file_path.clone(),
            ],
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspicious_exe_in_temp() {
        let monitor = FileMonitor::new();
        let event = FileEvent {
            ts: Utc::now(),
            file_path: "C:\\Users\\Admin\\AppData\\Local\\Temp\\payload.exe".to_string(),
            file_size: Some(50000),
            file_hash: None,
            action: FileAction::Created,
            pid: Some(1234),
            exe: Some("explorer.exe".to_string()),
            user: Some("admin".to_string()),
        };
        assert!(monitor.is_suspicious_file(&event));
    }

    #[test]
    fn test_double_extension() {
        let monitor = FileMonitor::new();
        let event = FileEvent {
            ts: Utc::now(),
            file_path: "C:\\Users\\Admin\\Documents\\invoice.pdf.exe".to_string(),
            file_size: Some(100000),
            file_hash: None,
            action: FileAction::Created,
            pid: Some(1234),
            exe: Some("explorer.exe".to_string()),
            user: Some("admin".to_string()),
        };
        assert!(monitor.is_suspicious_file(&event));
    }
}
