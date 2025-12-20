// windows/sensors/defender_adapter.rs
// Windows Defender + AMSI log adapter (auto-detect; degrade without error)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenderEvent {
    pub log_name: String,
    pub event_id: u32,
    pub ts: DateTime<Utc>,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct DefenderAdapter {
    available: bool,
}

impl Default for DefenderAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DefenderAdapter {
    pub fn new() -> Self {
        // Stub for now; real implementation would check registry/event log availability
        Self { available: false }
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Convert Defender/AMSI event to canonical core::Event
    pub fn to_canonical_event(&self, event: &DefenderEvent) -> Option<crate::Event> {
        match event.log_name.as_str() {
            "Microsoft-Windows-Windows Defender/Operational" => {
                match event.event_id {
                    1116 => {
                        // Detection (threat blocked)
                        let threat_name = event.fields.get("Threat Name").cloned();
                        let file = event.fields.get("File").cloned();

                        return Some(crate::Event {
                            ts: event.ts,
                            event_type: crate::event::EventType::Other,
                            tags: vec!["windows_defender".to_string(), "threat_detected".to_string(), threat_name.unwrap_or_default()],
                            ..Default::default()
                        });
                    },
                    1117 => {
                        // Threat action taken
                        return Some(crate::Event {
                            ts: event.ts,
                            event_type: crate::event::EventType::Other,
                            tags: vec!["windows_defender".to_string(), "threat_action".to_string()],
                            ..Default::default()
                        });
                    },
                    1121 => {
                        // Real-time protection disabled
                        return Some(crate::Event {
                            ts: event.ts,
                            event_type: crate::event::EventType::Other,
                            tags: vec!["windows_defender".to_string(), "defense_evasion".to_string(), "protection_disabled".to_string()],
                            ..Default::default()
                        });
                    },
                    _ => {}
                }
            },
            "Microsoft-Windows-Antimalware-Scan-Interface/Operational" => {
                match event.event_id {
                    1101 => {
                        // AMSI detection
                        let script = event.fields.get("Script").cloned();
                        return Some(crate::Event {
                            ts: event.ts,
                            event_type: crate::event::EventType::Other,
                            argv: script.map(|s| vec![s]),
                            tags: vec!["windows_amsi".to_string(), "script_detect".to_string()],
                            ..Default::default()
                        });
                    },
                    1102 => {
                        // AMSI scan started
                        return Some(crate::Event {
                            ts: event.ts,
                            event_type: crate::event::EventType::Other,
                            tags: vec!["windows_amsi".to_string(), "scan".to_string()],
                            ..Default::default()
                        });
                    },
                    _ => {}
                }
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
    fn test_defender_adapter_new() {
        let adapter = DefenderAdapter::new();
        assert!(!adapter.is_available());
    }
}
