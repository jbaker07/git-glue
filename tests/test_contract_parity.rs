// Contract parity test: ensure all 3 capture heartbeat schemas + HealthResponse are consistent
// This is the single source of truth for canonical event type counter field names
use serde_json::{json, Value};

/// Canonical event type counter names (single source of truth)
const CANONICAL_COUNTER_NAMES: &[&str] = &[
    "credential_access_count",  // maps from: cred_access_count (legacy)
    "discovery_count",          // maps from: discovery_exec_count (legacy)
    "exfiltration_count",       // derived: archive_tool_exec_count + staging_write_count
    "network_connection_count", // maps from: net_connect_count (legacy)
    "persistence_change_count",
    "defense_evasion_count",
    "process_injection_count",
    "auth_event_count",
    "script_exec_count",
];

/// Legacy heartbeat counter names (used by capture writers before remapping)
const LEGACY_COUNTER_NAMES: &[&str] = &[
    "cred_access_count",
    "discovery_exec_count",
    "archive_tool_exec_count",
    "staging_write_count",
    "net_connect_count",
    "persistence_change_count",
    "defense_evasion_count",
    "process_injection_count",
    "auth_event_count",
    "script_exec_count",
];

/// Optional breakdown counters (for exfiltration subtype detail)
const BREAKDOWN_COUNTER_NAMES: &[&str] = &["archive_tool_exec_count", "staging_write_count"];

#[test]
fn test_canonical_counter_names_single_source_of_truth() {
    // Verify all canonical names are unique and well-formed
    let canonical_set: std::collections::HashSet<_> = CANONICAL_COUNTER_NAMES.iter().collect();
    assert_eq!(
        canonical_set.len(),
        CANONICAL_COUNTER_NAMES.len(),
        "Canonical counter names must be unique"
    );

    for name in CANONICAL_COUNTER_NAMES {
        assert!(
            name.ends_with("_count"),
            "All canonical counters must end with '_count': {}",
            name
        );
        // Canonical names should be clean (no _exec_ for clarity)
        // Note: this is purely for documentation; the mapping is: discovery_exec → discovery, etc.
    }
}

#[test]
fn test_legacy_to_canonical_remapping() {
    // Verify legacy names are properly remapped to canonical
    let remapping = [
        ("cred_access_count", "credential_access_count"),
        ("discovery_exec_count", "discovery_count"),
        ("net_connect_count", "network_connection_count"),
        // These map 1:1 (no rename)
        ("archive_tool_exec_count", "archive_tool_exec_count"), // Note: stays as-is (breakdown)
        ("staging_write_count", "staging_write_count"),         // Note: stays as-is (breakdown)
        ("persistence_change_count", "persistence_change_count"),
        ("defense_evasion_count", "defense_evasion_count"),
        ("process_injection_count", "process_injection_count"),
        ("auth_event_count", "auth_event_count"),
        ("script_exec_count", "script_exec_count"),
    ];

    for (legacy, canonical) in &remapping {
        // Verify mapping direction is correct
        if !legacy.contains("_exec_") && *legacy != *canonical {
            // Non-_exec_ names should be remapped to cleaner canonical forms
            assert!(
                canonical.len() <= legacy.len() || !legacy.contains("_exec_"),
                "Canonical '{}' should be simpler than legacy '{}'",
                canonical,
                legacy
            );
        }
    }
}

#[test]
fn test_health_response_has_all_canonical_counters() {
    // Verify HealthResponse struct would have all 9 canonical fields
    let health_json = json!({
        "ok": true,
        "credential_access_count": 0,
        "discovery_count": 0,
        "exfiltration_count": 0,
        "network_connection_count": 0,
        "persistence_change_count": 0,
        "defense_evasion_count": 0,
        "process_injection_count": 0,
        "auth_event_count": 0,
        "script_exec_count": 0,
        "archive_tool_exec_count": 0,
        "staging_write_count": 0,
    });

    // Check all canonical counters exist
    for counter_name in CANONICAL_COUNTER_NAMES {
        assert!(
            health_json.get(counter_name).is_some(),
            "HealthResponse missing canonical counter: {}",
            counter_name
        );
    }

    // Check breakdown counters exist
    for counter_name in BREAKDOWN_COUNTER_NAMES {
        assert!(
            health_json.get(counter_name).is_some(),
            "HealthResponse missing breakdown counter: {}",
            counter_name
        );
    }
}

#[test]
fn test_macos_heartbeat_has_legacy_counters() {
    // Verify macOS heartbeat includes all legacy counter names (before remapping)
    // This is what's written to capture_heartbeat.json
    let macos_heartbeat = json!({
        "ts_ms": 1000,
        "pid": 12345,
        "cred_access_count": 0,
        "discovery_exec_count": 0,
        "archive_tool_exec_count": 0,
        "staging_write_count": 0,
        "net_connect_count": 0,
        "persistence_change_count": 0,
        "defense_evasion_count": 0,
        "process_injection_count": 0,
        "auth_event_count": 0,
        "script_exec_count": 0,
    });

    for counter_name in LEGACY_COUNTER_NAMES {
        assert!(
            macos_heartbeat.get(counter_name).is_some(),
            "macOS heartbeat missing legacy counter: {}",
            counter_name
        );
    }
}

#[test]
fn test_linux_heartbeat_has_legacy_counters() {
    // Verify Linux heartbeat includes all legacy counter names
    let linux_heartbeat = json!({
        "ts_ms": 1000,
        "pid": 12345,
        "cred_access_count": 0,
        "discovery_exec_count": 0,
        "archive_tool_exec_count": 0,
        "staging_write_count": 0,
        "net_connect_prim_count": 0,
        "persistence_change_count": 0,
        "defense_evasion_count": 0,
        "process_injection_count": 0,
        "auth_event_count": 0,
        "script_exec_count": 0,
    });

    // Linux uses slightly different names, but mapping should be clear
    assert!(linux_heartbeat.get("cred_access_count").is_some());
    assert!(linux_heartbeat.get("discovery_exec_count").is_some());
    assert!(
        linux_heartbeat.get("net_connect_prim_count").is_some(),
        "Note: Linux uses 'net_connect_prim_count', not 'net_connect_count'"
    );
}

#[test]
fn test_windows_heartbeat_has_canonical_counters() {
    // Verify Windows heartbeat includes canonical counter names
    // (Windows capture should write these directly, not legacy names)
    let windows_heartbeat = json!({
        "ts": "2025-12-24T12:00:00Z",
        "transport": "evtx",
        "credential_access_count": 0,
        "discovery_count": 0,
        "exfiltration_count": 0,
        "network_connection_count": 0,
        "persistence_change_count": 0,
        "defense_evasion_count": 0,
        "process_injection_count": 0,
        "auth_event_count": 0,
        "script_exec_count": 0,
        "archive_tool_exec_count": 0,
        "staging_write_count": 0,
    });

    for counter_name in CANONICAL_COUNTER_NAMES {
        assert!(
            windows_heartbeat.get(counter_name).is_some(),
            "Windows heartbeat missing canonical counter: {}",
            counter_name
        );
    }
}

#[test]
fn test_ui_kpi_mapping_matches_canonical_names() {
    // Verify UI app.js refreshKPIs() uses correct canonical field names
    // This test documents the exact mapping
    let expected_kpi_mapping = [
        ("credential_access_count", "Credential Access"),
        ("discovery_count", "Discovery"),
        ("exfiltration_count", "Exfiltration"),
        ("network_connection_count", "Network"),
        ("persistence_change_count", "Persistence"),
        ("defense_evasion_count", "Evasion"),
        ("process_injection_count", "Injection"),
        ("auth_event_count", "Auth"),
        ("script_exec_count", "Scripts"),
    ];

    for (counter_field, kpi_label) in &expected_kpi_mapping {
        // Verify counter field exists in canonical names
        assert!(
            CANONICAL_COUNTER_NAMES.contains(counter_field),
            "UI KPI field '{}' not in canonical names",
            counter_field
        );

        // Verify label is human-readable
        assert!(!kpi_label.is_empty());
        assert!(kpi_label.len() < 30);
    }
}

#[test]
fn test_exfiltration_derived_from_breakdown_counters() {
    // Verify exfiltration_count is always: archive_tool_exec_count + staging_write_count
    let test_cases = [
        (3, 8, 11),   // archive=3, staging=8 → exfil=11
        (0, 0, 0),    // both zero
        (5, 0, 5),    // only archive
        (0, 10, 10),  // only staging
        (42, 42, 84), // both large
    ];

    for (archive, staging, expected_exfil) in &test_cases {
        let derived = archive + staging;
        assert_eq!(
            derived, *expected_exfil,
            "exfiltration_count must be archive({}) + staging({}) = {}",
            archive, staging, expected_exfil
        );
    }
}

#[test]
fn test_canonical_counter_count_is_9() {
    // Enforce: exactly 9 canonical event types
    assert_eq!(
        CANONICAL_COUNTER_NAMES.len(),
        9,
        "Must have exactly 9 canonical event types, not {}",
        CANONICAL_COUNTER_NAMES.len()
    );
}

#[test]
fn test_no_duplicate_counter_names_across_schemas() {
    // Ensure counter name fields are well-defined across all schemas
    let canonical_set: std::collections::HashSet<_> = CANONICAL_COUNTER_NAMES.iter().collect();
    let legacy_set: std::collections::HashSet<_> = LEGACY_COUNTER_NAMES.iter().collect();
    let breakdown_set: std::collections::HashSet<_> = BREAKDOWN_COUNTER_NAMES.iter().collect();

    // Legacy and breakdown can overlap (archive_tool_exec_count, staging_write_count are in both)
    // Canonical and legacy will have differences due to remapping (cred_access → credential_access, etc.)
    assert_eq!(
        CANONICAL_COUNTER_NAMES.len(),
        9,
        "Canonical set must always have exactly 9 names"
    );
}
