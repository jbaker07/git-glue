// tests/windows_signals_integration.rs
// Integration tests for Windows signal detection

#[cfg(test)]
mod windows_signal_tests {
    use crate::core::event::{Event, EventType, EvidencePtr};
    use crate::windows::signal_engine::WindowsSignalEngine;
    use chrono::Utc;

    fn make_test_event(tags: Vec<&str>, proc_key: Option<String>) -> Event {
        let tags: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        Event {
            ts: Utc::now(),
            event_type: EventType::Other,
            proc_key,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 1,
            }),
            tags,
            ..Default::default()
        }
    }

    #[test]
    fn test_lsass_access_suspicious_fires() {
        let mut engine = WindowsSignalEngine::new("HOST1".to_string());
        let now = Utc::now().timestamp_millis();

        let event = Event {
            ts: Utc::now(),
            event_type: EventType::Ptrace,
            proc_key: Some("pid:200".to_string()),
            exe: Some("cmd.exe".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 1,
            }),
            tags: vec!["windows".to_string(), "credential_access".to_string(), "lsass_access".to_string()],
            ..Default::default()
        };

        let signals = engine.process_event(&event);
        assert!(!signals.is_empty(), "LSASSAccessSuspicious should fire");
        assert_eq!(signals[0].signal_type, "LSASSAccessSuspicious");
        assert_eq!(signals[0].severity, "critical");
    }

    #[test]
    fn test_wmi_persistence_fires() {
        let mut engine = WindowsSignalEngine::new("HOST1".to_string());

        let event = Event {
            ts: Utc::now(),
            event_type: EventType::Other,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 1,
            }),
            tags: vec!["windows".to_string(), "wmi".to_string(), "filter".to_string(), "persistence".to_string()],
            ..Default::default()
        };

        let signals = engine.process_event(&event);
        assert!(!signals.is_empty(), "WmiPersistenceConfirmed should fire");
        assert_eq!(signals[0].signal_type, "WmiPersistenceConfirmed");
        assert_eq!(signals[0].severity, "high");
    }

    #[test]
    fn test_log_evasion_fires() {
        let mut engine = WindowsSignalEngine::new("HOST1".to_string());

        // First event: log clear
        let log_clear = Event {
            ts: Utc::now(),
            event_type: EventType::Other,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 1,
            }),
            tags: vec!["windows".to_string(), "log_cleared".to_string(), "defense_evasion".to_string()],
            identity_key: Some("admin".to_string()),
            ..Default::default()
        };

        // Second event: audit policy change
        let audit_change = Event {
            ts: Utc::now(),
            event_type: EventType::Other,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 2,
            }),
            tags: vec!["windows".to_string(), "audit_policy".to_string(), "defense_evasion".to_string()],
            identity_key: Some("admin".to_string()),
            ..Default::default()
        };

        let _ = engine.process_event(&log_clear);
        let signals = engine.process_event(&audit_change);

        assert!(!signals.is_empty(), "LogEvasion should fire when log_clear + audit_change present");
        assert_eq!(signals[0].signal_type, "LogEvasion");
        assert_eq!(signals[0].severity, "critical");
    }

    #[test]
    fn test_lateral_share_access_fires() {
        let mut engine = WindowsSignalEngine::new("HOST1".to_string());

        let event = Event {
            ts: Utc::now(),
            event_type: EventType::Other,
            identity_key: Some("attacker".to_string()),
            file_key: Some("share:\\\\SERVER\\C$".to_string()),
            evidence_ptr: Some(EvidenceRef {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 1,
            }),
            tags: vec!["windows".to_string(), "network_access".to_string(), "lateral_movement".to_string()],
            ..Default::default()
        };

        let signals = engine.process_event(&event);
        assert!(!signals.is_empty(), "LateralShareAccess should fire on C$ access");
        assert_eq!(signals[0].signal_type, "LateralShareAccess");
        assert_eq!(signals[0].severity, "high");
    }

    #[test]
    fn test_remote_persistence_fires() {
        let mut engine = WindowsSignalEngine::new("HOST1".to_string());

        // First: remote logon
        let logon = Event {
            ts: Utc::now(),
            event_type: EventType::Other,
            identity_key: Some("admin".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 1,
            }),
            tags: vec!["windows".to_string(), "lateral_movement".to_string(), "remote_logon".to_string()],
            ..Default::default()
        };

        // Second: service installed
        let service = Event {
            ts: Utc::now(),
            event_type: EventType::Other,
            proc_key: Some("svc:TestService".to_string()),
            file_key: Some("path:C:\\malware.exe".to_string()),
            evidence_ptr: Some(EvidenceRef {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 2,
            }),
            tags: vec!["windows".to_string(), "service_installed".to_string(), "persistence".to_string()],
            ..Default::default()
        };

        let _ = engine.process_event(&logon);
        let signals = engine.process_event(&service);

        assert!(!signals.is_empty(), "RemotePersistence should fire");
        assert_eq!(signals[0].signal_type, "RemotePersistence");
        assert_eq!(signals[0].severity, "critical");
    }

    #[test]
    fn test_signal_dedup_cooldown() {
        let mut engine = WindowsSignalEngine::new("HOST1".to_string());

        let event = Event {
            ts: Utc::now(),
            event_type: EventType::Ptrace,
            exe: Some("cmd.exe".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "s1".to_string(),
                segment_id: "g1".to_string(),
                record_index: 1,
            }),
            tags: vec!["windows".to_string(), "credential_access".to_string(), "lsass_access".to_string()],
            ..Default::default()
        };

        let signals1 = engine.process_event(&event);
        assert!(!signals1.is_empty());

        // Second identical event should NOT fire (within cooldown)
        let signals2 = engine.process_event(&event);
        assert!(signals2.is_empty(), "Signal should be deduped within cooldown");
    }
}
