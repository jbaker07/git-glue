//! Contract tests for Linux primitive derivation
//!
//! Validates that canonical primitives are correctly derived from base sensor events
//! This test proves Linux routing: sensors_v2 → base events → derive_primitives → canonical
//!
//! **ROUTING PROOF:**
//! 1. Base event sources: linux/sensors_v2/mod.rs lines 52-78
//!    - procfs_process::collect(host)     → [linux/sensors_v2/procfs_process.rs:13-90]
//!    - process_monitor::collect(host)    → emits exec/exit events with pid, uid, euid, exe, argv
//!    - net_watch::collect(host)          → [linux/sensors_v2/net_watch.rs:12+] emits network events with remote_ip, remote_port
//!
//! 2. Event construction: linux/event_builders.rs
//!    - process_event() [lines 54-67]:    inserts PROC_PID, PROC_PPID, PROC_UID, PROC_EXE, PROC_ARGV via event_keys
//!    - network_event() [lines 69-81]:    inserts NET_REMOTE_IP, NET_REMOTE_PORT via event_keys::NET_*
//!
//! 3. Derivation: linux/capture_linux_rotating.rs:165
//!    - poll_and_write() calls sensors_v2::collect_all(host) → collects base events
//!    - passes each base event to derive_primitive_events(event) [line 166]
//!    - Linux detectors in linux/sensors/ebpf_primitives/ extract required fields using event_keys constants
//!
//! 4. Canonical primitives emitted:
//!    - credential_access:  ["pid", "uid", "euid", "exe", "cred_tool", "argv"]
//!    - discovery:          ["pid", "uid", "euid", "exe", "discovery_tool", "argv"]
//!    - exfiltration (archive): ["pid", "uid", "euid", "exe", "archive_tool", "primitive_subtype", "argv"]
//!    - exfiltration (staging): ["pid", "uid", "euid", "path", "op", "primitive_subtype"]
//!    - network_connection: ["pid", "uid", "euid", "remote_ip", "remote_port"]

#[cfg(test)]
mod tests {
    use forensic_hooks::core::{event_keys, Event};
    use forensic_hooks::linux::sensors::ebpf_primitives;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn create_base_exec_event(
        host: &str,
        exe: &str,
        argv: Vec<&str>,
        pid: u32,
        uid: u32,
        euid: u32,
    ) -> Event {
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(pid));
        fields.insert(event_keys::PROC_UID.to_string(), json!(uid));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(euid));
        fields.insert(event_keys::PROC_EXE.to_string(), json!(exe));
        fields.insert(event_keys::PROC_ARGV.to_string(), json!(argv));

        Event {
            ts_ms: 1000,
            host: host.to_string(),
            tags: vec![
                "linux".to_string(),
                "exec".to_string(),
                "ebpf_exec".to_string(),
            ],
            proc_key: None,
            file_key: None,
            identity_key: None,
            evidence_ptr: None,
            fields,
        }
    }

    fn create_base_network_event(
        host: &str,
        pid: u32,
        uid: u32,
        euid: u32,
        remote_ip: &str,
        remote_port: u16,
    ) -> Event {
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(pid));
        fields.insert(event_keys::PROC_UID.to_string(), json!(uid));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(euid));
        fields.insert(event_keys::NET_REMOTE_IP.to_string(), json!(remote_ip));
        fields.insert(event_keys::NET_REMOTE_PORT.to_string(), json!(remote_port));

        Event {
            ts_ms: 1000,
            host: host.to_string(),
            tags: vec![
                "linux".to_string(),
                "network".to_string(),
                "net_watch".to_string(),
            ],
            proc_key: None,
            file_key: None,
            identity_key: None,
            evidence_ptr: None,
            fields,
        }
    }

    fn create_base_file_event(
        host: &str,
        pid: u32,
        uid: u32,
        euid: u32,
        exe: &str,
        path: &str,
        op: &str,
    ) -> Event {
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(pid));
        fields.insert(event_keys::PROC_UID.to_string(), json!(uid));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(euid));
        fields.insert(event_keys::PROC_EXE.to_string(), json!(exe));
        fields.insert(event_keys::FILE_PATH.to_string(), json!(path));
        fields.insert(event_keys::FILE_OP.to_string(), json!(op));

        Event {
            ts_ms: 1000,
            host: host.to_string(),
            tags: vec![
                "linux".to_string(),
                "file".to_string(),
                "procfs_file".to_string(),
            ],
            proc_key: None,
            file_key: None,
            identity_key: None,
            evidence_ptr: None,
            fields,
        }
    }

    #[test]
    fn test_derive_credential_access_from_ssh_exec() {
        // Given: Base exec event for ssh binary
        let base = create_base_exec_event(
            "linux-host",
            "/usr/bin/ssh",
            vec!["/usr/bin/ssh", "user@example.com"],
            1234,
            1000,
            1000,
        );

        // When: Derive primitives
        let primitives = ebpf_primitives::derive_primitive_events(&base);

        // Then: Should emit credential_access primitive
        assert!(
            !primitives.is_empty(),
            "Should derive primitive from ssh exec"
        );

        let cred_access = primitives
            .iter()
            .find(|e| e.tags.get(1).map(|t| t.as_str()) == Some("credential_access"))
            .expect("Should emit credential_access primitive");

        // Validate contract: all required fields present with event_keys constants
        assert_eq!(
            cred_access
                .fields
                .get(event_keys::PROC_PID)
                .and_then(|v| v.as_u64()),
            Some(1234),
            "Must have canonical field: {}",
            event_keys::PROC_PID
        );
        assert_eq!(
            cred_access
                .fields
                .get(event_keys::PROC_UID)
                .and_then(|v| v.as_u64()),
            Some(1000),
            "Must have canonical field: {}",
            event_keys::PROC_UID
        );
        assert_eq!(
            cred_access
                .fields
                .get(event_keys::PROC_EUID)
                .and_then(|v| v.as_u64()),
            Some(1000),
            "Must have canonical field: {}",
            event_keys::PROC_EUID
        );
        assert_eq!(
            cred_access
                .fields
                .get(event_keys::PROC_EXE)
                .and_then(|v| v.as_str()),
            Some("/usr/bin/ssh"),
            "Must have canonical field: {}",
            event_keys::PROC_EXE
        );
        assert!(
            cred_access.fields.contains_key(event_keys::CRED_TOOL),
            "Must have canonical field: {}",
            event_keys::CRED_TOOL
        );
    }

    #[test]
    fn test_derive_discovery_from_ps_exec() {
        // Given: Base exec event for ps (process discovery)
        let base = create_base_exec_event(
            "linux-host",
            "/bin/ps",
            vec!["/bin/ps", "-aux"],
            2345,
            1000,
            1000,
        );

        // When: Derive primitives
        let primitives = ebpf_primitives::derive_primitive_events(&base);

        // Then: Should emit discovery primitive
        let discovery = primitives
            .iter()
            .find(|e| e.tags.get(1).map(|t| t.as_str()) == Some("discovery"))
            .expect("Should emit discovery primitive");

        // Validate contract
        assert_eq!(
            discovery
                .fields
                .get(event_keys::DISCOVERY_TOOL)
                .and_then(|v| v.as_str()),
            Some("ps"),
            "Must have canonical field with correct value: {}",
            event_keys::DISCOVERY_TOOL
        );
        assert!(
            discovery.fields.contains_key(event_keys::PROC_ARGV),
            "Must have canonical field: {}",
            event_keys::PROC_ARGV
        );
    }

    #[test]
    fn test_derive_archive_tool_exec_from_tar() {
        // Given: Base exec event for tar (exfiltration)
        let base = create_base_exec_event(
            "linux-host",
            "/bin/tar",
            vec!["/bin/tar", "-czf", "archive.tar.gz", "/home/user"],
            3456,
            1000,
            1000,
        );

        // When: Derive primitives
        let primitives = ebpf_primitives::derive_primitive_events(&base);

        // Then: Should emit exfiltration primitive with archive subtype
        let exfil = primitives
            .iter()
            .find(|e| e.tags.get(1).map(|t| t.as_str()) == Some("exfiltration"))
            .expect("Should emit exfiltration primitive");

        // CRITICAL CONTRACT: exfiltration MUST have primitive_subtype field
        assert_eq!(
            exfil
                .fields
                .get(event_keys::PRIMITIVE_SUBTYPE)
                .and_then(|v| v.as_str()),
            Some("archive_tool_exec"),
            "exfiltration must have primitive_subtype = 'archive_tool_exec'"
        );

        // Validate all required fields use event_keys constants
        assert!(
            exfil.fields.contains_key(event_keys::ARCHIVE_TOOL),
            "Must have canonical field: {}",
            event_keys::ARCHIVE_TOOL
        );
    }

    #[test]
    fn test_derive_staging_write_from_tmp_write() {
        // Given: Base file event for write to /tmp
        let base = create_base_file_event(
            "linux-host",
            4567,
            1000,
            1000,
            "/home/user/.ssh/keylogger",
            "/tmp/exfil_data.tar",
            "write",
        );

        // When: Derive primitives
        let primitives = ebpf_primitives::derive_primitive_events(&base);

        // Then: Should emit exfiltration primitive with staging subtype
        let exfil = primitives
            .iter()
            .find(|e| e.tags.get(1).map(|t| t.as_str()) == Some("exfiltration"))
            .expect("Should emit exfiltration primitive for /tmp write");

        // CRITICAL CONTRACT: exfiltration MUST have primitive_subtype field
        assert_eq!(
            exfil
                .fields
                .get(event_keys::PRIMITIVE_SUBTYPE)
                .and_then(|v| v.as_str()),
            Some("staging_write"),
            "exfiltration staging must have primitive_subtype = 'staging_write'"
        );

        // Validate all required fields use event_keys constants
        assert_eq!(
            exfil
                .fields
                .get(event_keys::FILE_PATH)
                .and_then(|v| v.as_str()),
            Some("/tmp/exfil_data.tar"),
            "Must have canonical field: {}",
            event_keys::FILE_PATH
        );
        assert_eq!(
            exfil
                .fields
                .get(event_keys::FILE_OP)
                .and_then(|v| v.as_str()),
            Some("write"),
            "Must have canonical field: {}",
            event_keys::FILE_OP
        );
    }

    #[test]
    fn test_derive_network_connection_from_external_ip() {
        // Given: Base network event for external connection
        let base = create_base_network_event(
            "linux-host",
            5678,
            1000,
            1000,
            "203.0.113.42", // External IP
            443,
        );

        // When: Derive primitives
        let primitives = ebpf_primitives::derive_primitive_events(&base);

        // Then: Should emit network_connection primitive
        let net_conn = primitives
            .iter()
            .find(|e| e.tags.get(1).map(|t| t.as_str()) == Some("network_connection"))
            .expect("Should emit network_connection primitive");

        // Validate contract: all required fields present with event_keys constants
        assert_eq!(
            net_conn
                .fields
                .get(event_keys::NET_REMOTE_IP)
                .and_then(|v| v.as_str()),
            Some("203.0.113.42"),
            "Must have canonical field: {}",
            event_keys::NET_REMOTE_IP
        );
        assert_eq!(
            net_conn
                .fields
                .get(event_keys::NET_REMOTE_PORT)
                .and_then(|v| v.as_u64()),
            Some(443),
            "Must have canonical field: {}",
            event_keys::NET_REMOTE_PORT
        );
    }

    #[test]
    fn test_validate_canonical_primitive_event_type() {
        // Given: Synthetic credential_access primitive
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(1234));
        fields.insert(event_keys::PROC_UID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EXE.to_string(), json!("/usr/bin/gpg"));
        fields.insert(event_keys::CRED_TOOL.to_string(), json!("gpg"));

        let event = Event {
            ts_ms: 1000,
            host: "test-host".to_string(),
            tags: vec![
                "linux".to_string(),
                "credential_access".to_string(),
                "ebpf".to_string(),
            ],
            proc_key: None,
            file_key: None,
            identity_key: None,
            evidence_ptr: None,
            fields,
        };

        // When: Validate canonical primitive
        let result = event.validate_canonical_primitive();

        // Then: Should pass validation
        assert!(
            result.is_ok(),
            "Should validate canonical primitive: {:?}",
            result
        );
    }

    #[test]
    fn test_reject_exfiltration_without_primitive_subtype() {
        // Given: exfiltration event WITHOUT primitive_subtype
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(1234));
        fields.insert(event_keys::PROC_UID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EXE.to_string(), json!("/bin/tar"));
        fields.insert(event_keys::ARCHIVE_TOOL.to_string(), json!("tar"));
        // NOTE: Missing PRIMITIVE_SUBTYPE field

        let event = Event {
            ts_ms: 1000,
            host: "test-host".to_string(),
            tags: vec![
                "linux".to_string(),
                "exfiltration".to_string(),
                "ebpf".to_string(),
            ],
            proc_key: None,
            file_key: None,
            identity_key: None,
            evidence_ptr: None,
            fields,
        };

        // When: Validate canonical primitive
        let result = event.validate_canonical_primitive();

        // Then: Should fail - exfiltration MUST have primitive_subtype
        assert!(
            result.is_err(),
            "Should reject exfiltration without primitive_subtype"
        );
        assert!(
            result.unwrap_err().contains("primitive_subtype"),
            "Error message should mention primitive_subtype"
        );
    }

    #[test]
    fn test_reject_invalid_event_type() {
        // Given: Event with invalid event_type in tags[1]
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(1234));

        let event = Event {
            ts_ms: 1000,
            host: "test-host".to_string(),
            tags: vec!["linux".to_string(), "invalid_type".to_string()], // Invalid!
            proc_key: None,
            file_key: None,
            identity_key: None,
            evidence_ptr: None,
            fields,
        };

        // When: Validate canonical primitive
        let result = event.validate_canonical_primitive();

        // Then: Should fail - only canonical types allowed
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Event type must be one of"));
    }
}
