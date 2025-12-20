// windows/sensors/process_monitor.rs
// Process behavior monitoring (injection, token abuse, privilege escalation)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub ts: DateTime<Utc>,
    pub event_type: ProcessEventType,
    pub pid: i32,
    pub ppid: Option<i32>,
    pub exe: String,
    pub parent_exe: Option<String>,
    pub integrity_level: Option<String>,  // Low, Medium, High, System
    pub user: Option<String>,
    pub command_line: Option<String>,
    pub handles_opened: Option<Vec<String>>,  // Processes it opened handles to
    pub token_privilege: Option<Vec<String>>,  // SeDebug, SeImpersonate, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessEventType {
    Created,
    Terminated,
    HandleOpened,
    TokenPrivilege,
    MemoryInjection,
    CodeCave,
}

#[derive(Debug)]
pub struct ProcessMonitor;

impl ProcessMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn is_suspicious_process_behavior(&self, event: &ProcessEvent) -> bool {
        match event.event_type {
            ProcessEventType::MemoryInjection => true,
            ProcessEventType::CodeCave => true,
            
            ProcessEventType::HandleOpened => {
                // Opening handle to system processes is suspicious
                if let Some(handles) = &event.handles_opened {
                    for handle in handles {
                        if handle.to_lowercase().contains("lsass") ||
                           handle.to_lowercase().contains("system") ||
                           handle.to_lowercase().contains("csrss") {
                            return true;
                        }
                    }
                }
                false
            },
            
            ProcessEventType::TokenPrivilege => {
                // High-risk token privileges
                if let Some(privs) = &event.token_privilege {
                    for priv_name in privs {
                        let lower = priv_name.to_lowercase();
                        if lower.contains("sedebug") ||
                           lower.contains("seimpersonate") ||
                           lower.contains("seassignprimarytoken") ||
                           lower.contains("setcb") {
                            return true;
                        }
                    }
                }
                false
            },
            
            ProcessEventType::Created => {
                // Parent-child anomalies
                if let Some(parent) = &event.parent_exe {
                    let lower_parent = parent.to_lowercase();
                    
                    // Explorer spawning cmd/powershell
                    if (lower_parent.contains("explorer.exe") || 
                        lower_parent.contains("notepad.exe")) &&
                       (event.exe.to_lowercase().contains("cmd.exe") ||
                        event.exe.to_lowercase().contains("powershell.exe")) {
                        return true;
                    }
                    
                    // Services.exe spawning unusual processes
                    if lower_parent.contains("services.exe") &&
                       !is_legitimate_service_child(&event.exe) {
                        return true;
                    }
                }
                
                // User to SYSTEM privilege escalation
                if let (Some(user), Some(integrity)) = (&event.user, &event.integrity_level) {
                    if !user.contains("SYSTEM") && integrity == "System" {
                        return true;
                    }
                }
                
                false
            },
            
            _ => false,
        }
    }

    pub fn to_canonical_event(&self, event: &ProcessEvent) -> Option<crate::Event> {
        if !self.is_suspicious_process_behavior(event) {
            return None;
        }

        Some(crate::Event {
            ts: event.ts,
            event_type: crate::event::EventType::Execve,
            pid: Some(event.pid),
            ppid: event.ppid,
            exe: Some(event.exe.clone()),
            argv: event.command_line.as_ref().map(|c| vec![c.clone()]),
            tags: vec![
                "windows_process".to_string(),
                format!("{:?}", event.event_type).to_lowercase(),
                event.integrity_level.clone().unwrap_or_default(),
            ],
            ..Default::default()
        })
    }
}

fn is_legitimate_service_child(exe: &str) -> bool {
    let lower = exe.to_lowercase();
    lower.contains("svchost.exe") ||
    lower.contains("rundll32.exe") ||
    lower.contains("taskeng.exe") ||
    lower.contains("taskhostw.exe")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explorer_spawning_cmd() {
        let monitor = ProcessMonitor::new();
        let event = ProcessEvent {
            ts: Utc::now(),
            event_type: ProcessEventType::Created,
            pid: 1234,
            ppid: Some(500),
            exe: "cmd.exe".to_string(),
            parent_exe: Some("explorer.exe".to_string()),
            integrity_level: Some("Medium".to_string()),
            user: Some("admin".to_string()),
            command_line: None,
            handles_opened: None,
            token_privilege: None,
        };
        assert!(monitor.is_suspicious_process_behavior(&event));
    }

    #[test]
    fn test_token_sedebug() {
        let monitor = ProcessMonitor::new();
        let event = ProcessEvent {
            ts: Utc::now(),
            event_type: ProcessEventType::TokenPrivilege,
            pid: 1234,
            ppid: None,
            exe: "malware.exe".to_string(),
            parent_exe: None,
            integrity_level: None,
            user: None,
            command_line: None,
            handles_opened: None,
            token_privilege: Some(vec!["SeDebug".to_string()]),
        };
        assert!(monitor.is_suspicious_process_behavior(&event));
    }
}
