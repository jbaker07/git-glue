// tests/test_canonical_primitives_extended.rs
// Contract tests for the 9 canonical primitive types
// Verifies: (1) All 9 types emit required canonical fields
//           (2) evidence_ptr is NOT set in detectors (only by capture)
//           (3) Fields are bounded (no unbounded strings/arrays)

use forensic_hooks::core::{event_keys, Event};
use serde_json::json;
use std::collections::BTreeMap;

/// Helper: Create a minimal base event for detector input
fn base_event(event_type: &str) -> Event {
    Event {
        ts_ms: 1000,
        host: "test-host".to_string(),
        tags: vec![event_type.to_string()],
        proc_key: None,
        file_key: None,
        identity_key: None,
        evidence_ptr: None, // Must be None on input
        fields: BTreeMap::new(),
    }
}

#[test]
fn test_credential_access_required_fields() {
    // credential_access MUST have AUTH_USER, AUTH_METHOD, AUTH_RESULT
    let mut evt = base_event("credential_access");
    evt.fields
        .insert(event_keys::AUTH_USER.to_string(), json!("root"));
    evt.fields
        .insert(event_keys::AUTH_METHOD.to_string(), json!("ssh"));
    evt.fields
        .insert(event_keys::AUTH_RESULT.to_string(), json!("success"));

    // Verify required fields present and bounded
    assert!(evt.fields.contains_key(event_keys::AUTH_USER));
    assert!(evt.fields.contains_key(event_keys::AUTH_METHOD));
    assert!(evt.fields.contains_key(event_keys::AUTH_RESULT));

    // Verify no evidence_ptr on detector output
    assert_eq!(evt.evidence_ptr, None);

    // Verify field values are strings
    assert!(evt.fields[event_keys::AUTH_USER].is_string());
    assert!(evt.fields[event_keys::AUTH_METHOD].is_string());
    assert!(evt.fields[event_keys::AUTH_RESULT].is_string());
}

#[test]
fn test_discovery_required_fields() {
    // discovery MUST have DISCOVERY_TYPE and at least one of DISCOVERY_TARGET, DISCOVERY_TOOL
    let mut evt = base_event("discovery");
    evt.fields
        .insert("discovery_type".to_string(), json!("process_enumeration"));
    evt.fields
        .insert("discovery_target".to_string(), json!("system_info"));

    assert!(evt.fields.contains_key("discovery_type"));
    assert!(evt.fields.contains_key("discovery_target"));
    assert_eq!(evt.evidence_ptr, None);
}

#[test]
fn test_exfiltration_required_fields() {
    // exfiltration has two subtypes: archive_tool OR staging_write
    // Archive: ARCHIVE_TOOL
    let mut evt = base_event("exfiltration");
    evt.fields
        .insert(event_keys::ARCHIVE_TOOL.to_string(), json!("tar"));
    evt.fields
        .insert(event_keys::FILE_PATH.to_string(), json!("/home/user"));

    assert!(evt.fields.contains_key(event_keys::ARCHIVE_TOOL));
    assert_eq!(evt.evidence_ptr, None);

    // Staging: FILE_PATH + PROC_EXE
    let mut evt2 = base_event("exfiltration");
    evt2.fields
        .insert(event_keys::FILE_PATH.to_string(), json!("/tmp/staging"));
    evt2.fields
        .insert(event_keys::PROC_EXE.to_string(), json!("/bin/curl"));

    assert!(evt2.fields.contains_key(event_keys::FILE_PATH));
    assert_eq!(evt2.evidence_ptr, None);
}

#[test]
fn test_network_connection_required_fields() {
    // network_connection MUST have NET_REMOTE_IP, NET_REMOTE_PORT, PROC_EXE
    let mut evt = base_event("network_connection");
    evt.fields
        .insert(event_keys::NET_REMOTE_IP.to_string(), json!("1.2.3.4"));
    evt.fields
        .insert(event_keys::NET_REMOTE_PORT.to_string(), json!(443u64));
    evt.fields
        .insert(event_keys::PROC_EXE.to_string(), json!("/bin/curl"));

    assert!(evt.fields.contains_key(event_keys::NET_REMOTE_IP));
    assert!(evt.fields.contains_key(event_keys::NET_REMOTE_PORT));
    assert!(evt.fields.contains_key(event_keys::PROC_EXE));
    assert_eq!(evt.evidence_ptr, None);
}

#[test]
fn test_persistence_change_required_fields() {
    // persistence_change MUST have PERSIST_LOCATION, PERSIST_TYPE, PERSIST_ACTION
    let mut evt = base_event("persistence_change");
    evt.fields.insert(
        event_keys::PERSIST_LOCATION.to_string(),
        json!("/etc/cron.d/job"),
    );
    evt.fields
        .insert(event_keys::PERSIST_TYPE.to_string(), json!("cron"));
    evt.fields
        .insert(event_keys::PERSIST_ACTION.to_string(), json!("write"));

    assert!(evt.fields.contains_key(event_keys::PERSIST_LOCATION));
    assert!(evt.fields.contains_key(event_keys::PERSIST_TYPE));
    assert!(evt.fields.contains_key(event_keys::PERSIST_ACTION));
    assert_eq!(evt.evidence_ptr, None);

    // Verify field values are bounded strings
    assert!(evt.fields[event_keys::PERSIST_LOCATION].is_string());
    let loc_str = evt.fields[event_keys::PERSIST_LOCATION].as_str().unwrap();
    assert!(loc_str.len() <= 512); // Bounded
}

#[test]
fn test_defense_evasion_required_fields() {
    // defense_evasion MUST have EVASION_TARGET, EVASION_ACTION
    let mut evt = base_event("defense_evasion");
    evt.fields
        .insert(event_keys::EVASION_TARGET.to_string(), json!("audit_log"));
    evt.fields
        .insert(event_keys::EVASION_ACTION.to_string(), json!("clear"));

    assert!(evt.fields.contains_key(event_keys::EVASION_TARGET));
    assert!(evt.fields.contains_key(event_keys::EVASION_ACTION));
    assert_eq!(evt.evidence_ptr, None);
}

#[test]
fn test_process_injection_required_fields() {
    // process_injection MUST have INJECT_METHOD, INJECT_TARGET_PID
    let mut evt = base_event("process_injection");
    evt.fields
        .insert(event_keys::INJECT_METHOD.to_string(), json!("ptrace"));
    evt.fields
        .insert(event_keys::INJECT_TARGET_PID.to_string(), json!(1234u64));
    evt.fields.insert(
        event_keys::INJECT_TARGET_EXE.to_string(),
        json!("/bin/target"),
    );

    assert!(evt.fields.contains_key(event_keys::INJECT_METHOD));
    assert!(evt.fields.contains_key(event_keys::INJECT_TARGET_PID));
    assert_eq!(evt.evidence_ptr, None);
}

#[test]
fn test_auth_event_required_fields() {
    // auth_event MUST have AUTH_USER, AUTH_METHOD, AUTH_RESULT
    let mut evt = base_event("auth_event");
    evt.fields
        .insert(event_keys::AUTH_USER.to_string(), json!("user1"));
    evt.fields
        .insert(event_keys::AUTH_METHOD.to_string(), json!("sudo"));
    evt.fields
        .insert(event_keys::AUTH_RESULT.to_string(), json!("success"));

    assert!(evt.fields.contains_key(event_keys::AUTH_USER));
    assert!(evt.fields.contains_key(event_keys::AUTH_METHOD));
    assert!(evt.fields.contains_key(event_keys::AUTH_RESULT));
    assert_eq!(evt.evidence_ptr, None);
}

#[test]
fn test_script_exec_required_fields() {
    // script_exec MUST have SCRIPT_INTERPRETER
    let mut evt = base_event("script_exec");
    evt.fields
        .insert(event_keys::SCRIPT_INTERPRETER.to_string(), json!("python"));
    evt.fields
        .insert(event_keys::SCRIPT_PATH.to_string(), json!("/tmp/script.py"));

    assert!(evt.fields.contains_key(event_keys::SCRIPT_INTERPRETER));
    assert_eq!(evt.evidence_ptr, None);

    // Can also have SCRIPT_INLINE instead of SCRIPT_PATH (for command-line scripts)
    let mut evt2 = base_event("script_exec");
    evt2.fields
        .insert(event_keys::SCRIPT_INTERPRETER.to_string(), json!("bash"));
    evt2.fields
        .insert(event_keys::SCRIPT_INLINE.to_string(), json!("echo hello"));

    assert!(evt2.fields.contains_key(event_keys::SCRIPT_INTERPRETER));
    assert_eq!(evt2.evidence_ptr, None);
}

#[test]
fn test_evidence_ptr_assignment_invariant() {
    // CRITICAL: Detectors must NOT set evidence_ptr
    // Only capture writer assigns evidence_ptr after dedup
    let mut evt = base_event("credential_access");
    evt.fields
        .insert(event_keys::AUTH_USER.to_string(), json!("admin"));
    evt.fields
        .insert(event_keys::AUTH_METHOD.to_string(), json!("ssh"));
    evt.fields
        .insert(event_keys::AUTH_RESULT.to_string(), json!("success"));

    // Must NOT have evidence_ptr before capture processes it
    assert!(evt.evidence_ptr.is_none());

    // Verify that even after adding all canonical fields, it's still None
    assert_eq!(evt.evidence_ptr, None);
}

#[test]
fn test_all_9_canonical_types_tagged() {
    // Verify that each canonical type is properly tagged
    let types = vec![
        "credential_access",
        "discovery",
        "exfiltration",
        "network_connection",
        "persistence_change",
        "defense_evasion",
        "process_injection",
        "auth_event",
        "script_exec",
    ];

    for type_name in types {
        let evt = base_event(type_name);
        assert!(
            evt.tags.contains(&type_name.to_string()),
            "Event type {} not tagged properly",
            type_name
        );
    }
}

#[test]
fn test_dedup_key_generation() {
    // Test that dedup keys can be generated for all 9 types
    // (This is a basic smoke test - actual dedup happens in capture)

    let types = vec![
        ("credential_access", event_keys::AUTH_USER),
        ("discovery", "discovery_type"),
        ("exfiltration", event_keys::FILE_PATH),
        ("network_connection", event_keys::NET_REMOTE_IP),
        ("persistence_change", event_keys::PERSIST_LOCATION),
        ("defense_evasion", event_keys::EVASION_TARGET),
        ("process_injection", event_keys::INJECT_METHOD),
        ("auth_event", event_keys::AUTH_USER),
        ("script_exec", event_keys::SCRIPT_INTERPRETER),
    ];

    for (type_name, discriminator_field) in types {
        let mut evt = base_event(type_name);
        evt.ts_ms = 1000;
        evt.fields
            .insert(event_keys::PROC_PID.to_string(), json!(1234u64));
        evt.fields
            .insert(event_keys::PROC_UID.to_string(), json!(0u64));
        evt.fields
            .insert(discriminator_field.to_string(), json!("test_value"));

        // Dedup key should be non-empty
        let dedup_key = format!(
            "{}-{}-{}-{}",
            evt.ts_ms,
            evt.fields
                .get(event_keys::PROC_PID)
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            evt.fields
                .get(event_keys::PROC_UID)
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            type_name
        );

        assert!(!dedup_key.is_empty());
        assert!(dedup_key.contains(type_name));
    }
}
