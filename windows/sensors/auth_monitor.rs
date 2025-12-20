// windows/sensors/auth_monitor.rs
// Authentication and logon monitoring (anomalies, brute force, pass-the-hash)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEvent {
    pub ts: DateTime<Utc>,
    pub event_type: AuthEventType,
    pub account: String,
    pub domain: Option<String>,
    pub logon_type: Option<LogonType>,
    pub source_ip: Option<String>,
    pub source_hostname: Option<String>,
    pub success: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthEventType {
    LogonSuccess,
    LogonFailure,
    LogoffSuccess,
    Kerberos,
    Ntlm,
    LocalAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogonType {
    Interactive,       // 2
    Network,           // 3
    Batch,             // 4
    Service,           // 5
    Unlock,            // 7
    NetworkCleartext,  // 8
    NewCredentials,    // 9
    RemoteInteractive, // 10
    CachedInteractive, // 11
}

#[derive(Debug)]
pub struct AuthMonitor {
    failure_cache: HashMap<String, (u32, DateTime<Utc>)>, // account -> (count, first_failure_time)
}

impl AuthMonitor {
    pub fn new() -> Self {
        Self {
            failure_cache: HashMap::new(),
        }
    }

    pub fn is_suspicious_auth(&mut self, event: &AuthEvent) -> bool {
        if !event.success {
            // Track failed attempts
            let entry = self.failure_cache.entry(event.account.clone())
                .or_insert((0, event.ts));
            entry.0 += 1;

            // More than 5 failures in 5 minutes = brute force
            let duration = event.ts.signed_duration_since(entry.1);
            if entry.0 > 5 && duration.num_minutes() <= 5 {
                return true;
            }

            return false;
        }

        // Success after recent failures = suspicious
        if let Some((count, first_failure)) = self.failure_cache.get(&event.account) {
            if *count >= 3 {
                let duration = event.ts.signed_duration_since(*first_failure);
                if duration.num_minutes() <= 5 {
                    return true;
                }
            }
        }

        // Remote logon from unusual location
        if let Some(logon_type) = &event.logon_type {
            if matches!(logon_type, LogonType::RemoteInteractive | LogonType::Network) {
                if let Some(ip) = &event.source_ip {
                    // Non-private network remote logon is suspicious
                    if !is_private_ip(ip) && !is_localhost(ip) {
                        return true;
                    }
                }
            }
        }

        // Network logon as SYSTEM/LocalSystem (unusual)
        if matches!(event.logon_type, Some(LogonType::Network)) {
            if event.account.contains("SYSTEM") || event.account.contains("LocalSystem") {
                return true;
            }
        }

        false
    }

    pub fn to_canonical_event(&self, event: &AuthEvent) -> Option<crate::Event> {
        Some(crate::Event {
            ts: event.ts,
            event_type: crate::event::EventType::Other,
            tags: vec![
                "windows_auth".to_string(),
                format!("{:?}", event.event_type).to_lowercase(),
                format!("success_{}", event.success),
                event.account.clone(),
                event.source_ip.clone().unwrap_or_default(),
            ],
            ..Default::default()
        })
    }
}

impl Default for AuthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

fn is_private_ip(ip: &str) -> bool {
    ip.starts_with("10.") || 
    ip.starts_with("172.16.") || 
    ip.starts_with("192.168.") ||
    ip.starts_with("169.254.")
}

fn is_localhost(ip: &str) -> bool {
    ip == "127.0.0.1" || ip == "::1" || ip == "localhost"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brute_force_detection() {
        let mut monitor = AuthMonitor::new();
        let base_time = Utc::now();

        // Simulate 6 failed attempts in 3 minutes
        for i in 0..6 {
            let event = AuthEvent {
                ts: base_time + chrono::Duration::minutes(i as i64),
                event_type: AuthEventType::LogonFailure,
                account: "admin".to_string(),
                domain: None,
                logon_type: None,
                source_ip: Some("192.168.1.10".to_string()),
                source_hostname: None,
                success: false,
                reason: Some("Invalid credentials".to_string()),
            };
            if i == 5 {
                assert!(monitor.is_suspicious_auth(&event));
            }
        }
    }
}
