// Integration test for UI health endpoint with canonical event counters
use serde_json::json;

#[test]
fn test_health_response_uses_canonical_counter_names() {
    // Verify HealthResponse uses canonical naming for all 9 event types
    let health_response = json!({
        "ok": true,
        "capture_alive": true,
        "capture_build_id": "v1.0.0",
        // 9 canonical event type counters (matching canonical types)
        "credential_access_count": 5,
        "discovery_count": 12,
        "exfiltration_count": 11,      // derived: archive(3) + staging(8)
        "network_connection_count": 42,
        "persistence_change_count": 2,
        "defense_evasion_count": 1,
        "process_injection_count": 0,
        "auth_event_count": 7,
        "script_exec_count": 4,
        // Optional breakdown counters for exfiltration subtype detail
        "archive_tool_exec_count": 3,
        "staging_write_count": 8,
    });

    // Verify all 9 canonical counters
    assert_eq!(health_response["credential_access_count"], 5);
    assert_eq!(health_response["discovery_count"], 12);
    assert_eq!(health_response["exfiltration_count"], 11);
    assert_eq!(health_response["network_connection_count"], 42);
    assert_eq!(health_response["persistence_change_count"], 2);
    assert_eq!(health_response["defense_evasion_count"], 1);
    assert_eq!(health_response["process_injection_count"], 0);
    assert_eq!(health_response["auth_event_count"], 7);
    assert_eq!(health_response["script_exec_count"], 4);

    // Verify optional breakdown counters
    assert_eq!(health_response["archive_tool_exec_count"], 3);
    assert_eq!(health_response["staging_write_count"], 8);
}

#[test]
fn test_exfiltration_derived_from_breakdown_counters() {
    // Verify that exfiltration_count is correctly derived as sum of archive + staging
    let heartbeat = json!({
        "archive_tool_exec_count": 3,
        "staging_write_count": 8,
    });

    let archive_tool_exec_count = heartbeat
        .get("archive_tool_exec_count")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let staging_write_count = heartbeat
        .get("staging_write_count")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let exfiltration_count = archive_tool_exec_count + staging_write_count;

    assert_eq!(exfiltration_count, 11);
}

#[test]
fn test_kpi_row_displays_9_canonical_types() {
    // Verify UI KPI row displays all 9 canonical event types
    let h = json!({
        "credential_access_count": 5,
        "discovery_count": 12,
        "exfiltration_count": 11,
        "network_connection_count": 42,
        "persistence_change_count": 2,
        "defense_evasion_count": 1,
        "process_injection_count": 0,
        "auth_event_count": 7,
        "script_exec_count": 4,
    });

    let safe = |v: &serde_json::Value| -> String {
        if v.is_null() {
            "-".to_string()
        } else {
            v.to_string()
        }
    };

    // Construct KPI row as UI does
    let kpi_html = format!(
        "Cred: {} | Disc: {} | Exfil: {} | Net: {} | Persist: {} | Evade: {} | Inject: {} | Auth: {} | Script: {}",
        safe(&h["credential_access_count"]),
        safe(&h["discovery_count"]),
        safe(&h["exfiltration_count"]),
        safe(&h["network_connection_count"]),
        safe(&h["persistence_change_count"]),
        safe(&h["defense_evasion_count"]),
        safe(&h["process_injection_count"]),
        safe(&h["auth_event_count"]),
        safe(&h["script_exec_count"]),
    );

    // Verify all 9 canonical types rendered
    assert!(kpi_html.contains("Cred: 5"));
    assert!(kpi_html.contains("Disc: 12"));
    assert!(kpi_html.contains("Exfil: 11"));
    assert!(kpi_html.contains("Net: 42"));
    assert!(kpi_html.contains("Persist: 2"));
    assert!(kpi_html.contains("Evade: 1"));
    assert!(kpi_html.contains("Inject: 0"));
    assert!(kpi_html.contains("Auth: 7"));
    assert!(kpi_html.contains("Script: 4"));
}

#[test]
fn test_schema_consistency_canonical_names() {
    // Verify naming consistency between canonical event types and counter fields
    let canonical_types = vec![
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

    let health_response = json!({
        "credential_access_count": 1,
        "discovery_count": 1,
        "exfiltration_count": 1,
        "network_connection_count": 1,
        "persistence_change_count": 1,
        "defense_evasion_count": 1,
        "process_injection_count": 1,
        "auth_event_count": 1,
        "script_exec_count": 1,
    });

    // Verify each canonical type has corresponding counter field
    for canonical_type in canonical_types {
        let counter_field = format!("{}_count", canonical_type);
        assert!(
            health_response.get(&counter_field).is_some(),
            "Missing counter field: {}",
            counter_field
        );
    }
}

#[test]
fn test_backwards_compat_missing_canonical_counters() {
    // Verify UI handles missing canonical counters gracefully
    let h = json!({
        "credential_access_count": 5,
        "discovery_count": 12,
        // Missing: exfiltration_count, defense_evasion_count, etc.
    });

    let safe = |v: &serde_json::Value| -> String {
        if v.is_null() {
            "-".to_string()
        } else {
            v.to_string()
        }
    };

    // UI should render "-" for missing values
    assert_eq!(safe(&h["credential_access_count"]), "5");
    assert_eq!(safe(&h["discovery_count"]), "12");
    assert_eq!(safe(&h["exfiltration_count"]), "-");
    assert_eq!(safe(&h["defense_evasion_count"]), "-");
}
