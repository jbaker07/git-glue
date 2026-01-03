/// Generate canonical JSON examples for each primitive type (both platforms)
/// This demonstrates the locked schema contract with all required fields present

#[cfg(test)]
mod tests {
    use forensic_hooks::core::event_keys;
    use forensic_hooks::core::Event;
    use forensic_hooks::core::EvidencePtr;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    /// Serialize Event to canonical JSON following the locked schema
    fn event_to_json(event: &Event) -> Value {
        let fields_value: BTreeMap<String, Value> = event
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        json!({
            "ts_ms": event.ts_ms,
            "host": event.host,
            "tags": event.tags,
            "proc_key": event.proc_key,
            "file_key": event.file_key,
            "identity_key": event.identity_key,
            "evidence_ptr": event.evidence_ptr.as_ref().map(|ptr| {
                json!({
                    "stream_id": &ptr.stream_id,
                    "segment_id": ptr.segment_id,
                    "record_index": ptr.record_index,
                })
            }),
            "fields": fields_value,
        })
    }

    #[test]
    fn example_credential_access_canonical_json() {
        // Canonical credential_access event (ssh execution)
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(2847));
        fields.insert(event_keys::PROC_UID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EXE.to_string(), json!("/usr/bin/ssh"));
        fields.insert(
            event_keys::PROC_ARGV.to_string(),
            json!(["ssh", "user@192.0.2.1"]),
        );
        fields.insert(event_keys::CRED_TOOL.to_string(), json!("ssh"));

        let event = Event {
            ts_ms: 1704067200000,
            host: "production-web-01".to_string(),
            tags: vec![
                "linux".to_string(),
                "credential_access".to_string(),
                "ebpf".to_string(),
            ],
            proc_key: Some("host:prod-web-01:pid:2847".to_string()),
            file_key: None,
            identity_key: Some("host:prod-web-01:uid:1000".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "linux_capture_0".to_string(),
                segment_id: 42,
                record_index: 156,
            }),
            fields,
        };

        let json_out = event_to_json(&event);
        println!("=== CANONICAL: credential_access ===");
        println!("{}", serde_json::to_string_pretty(&json_out).unwrap());

        // Verify schema contract
        assert_eq!(json_out["tags"][1], "credential_access");
        assert!(json_out["fields"][event_keys::PROC_PID].is_number());
        assert!(json_out["fields"][event_keys::CRED_TOOL].is_string());
    }

    #[test]
    fn example_discovery_canonical_json() {
        // Canonical discovery event (ps execution)
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(3156));
        fields.insert(event_keys::PROC_UID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EXE.to_string(), json!("/bin/ps"));
        fields.insert(event_keys::PROC_ARGV.to_string(), json!(["ps", "-aux"]));
        fields.insert(event_keys::DISCOVERY_TOOL.to_string(), json!("ps"));

        let event = Event {
            ts_ms: 1704067201000,
            host: "production-app-02".to_string(),
            tags: vec![
                "linux".to_string(),
                "discovery".to_string(),
                "ebpf".to_string(),
            ],
            proc_key: Some("host:prod-app-02:pid:3156".to_string()),
            file_key: None,
            identity_key: Some("host:prod-app-02:uid:1000".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "linux_capture_0".to_string(),
                segment_id: 42,
                record_index: 157,
            }),
            fields,
        };

        let json_out = event_to_json(&event);
        println!("=== CANONICAL: discovery ===");
        println!("{}", serde_json::to_string_pretty(&json_out).unwrap());

        assert_eq!(json_out["tags"][1], "discovery");
        assert_eq!(json_out["fields"][event_keys::DISCOVERY_TOOL], "ps");
    }

    #[test]
    fn example_exfiltration_archive_canonical_json() {
        // Canonical exfiltration primitive: archive_tool_exec (tar)
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(4521));
        fields.insert(event_keys::PROC_UID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EXE.to_string(), json!("/bin/tar"));
        fields.insert(
            event_keys::PROC_ARGV.to_string(),
            json!(["tar", "-czf", "archive.tar.gz", "/var/www/html"]),
        );
        fields.insert(event_keys::ARCHIVE_TOOL.to_string(), json!("tar"));
        fields.insert(
            event_keys::PRIMITIVE_SUBTYPE.to_string(),
            json!("archive_tool_exec"),
        );

        let event = Event {
            ts_ms: 1704067202000,
            host: "production-db-03".to_string(),
            tags: vec![
                "linux".to_string(),
                "exfiltration".to_string(),
                "ebpf".to_string(),
            ],
            proc_key: Some("host:prod-db-03:pid:4521".to_string()),
            file_key: None,
            identity_key: Some("host:prod-db-03:uid:1000".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "linux_capture_0".to_string(),
                segment_id: 43,
                record_index: 284,
            }),
            fields,
        };

        let json_out = event_to_json(&event);
        println!("=== CANONICAL: exfiltration (archive_tool_exec) ===");
        println!("{}", serde_json::to_string_pretty(&json_out).unwrap());

        assert_eq!(json_out["tags"][1], "exfiltration");
        assert_eq!(
            json_out["fields"][event_keys::PRIMITIVE_SUBTYPE],
            "archive_tool_exec"
        );
        assert!(json_out["fields"][event_keys::ARCHIVE_TOOL].is_string());
    }

    #[test]
    fn example_exfiltration_staging_canonical_json() {
        // Canonical exfiltration primitive: staging_write (/tmp)
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(5897));
        fields.insert(event_keys::PROC_UID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(1000));
        fields.insert(
            event_keys::PROC_EXE.to_string(),
            json!("/usr/local/bin/data_processor"),
        );
        fields.insert(
            event_keys::FILE_PATH.to_string(),
            json!("/tmp/output_2024.data"),
        );
        fields.insert(event_keys::FILE_OP.to_string(), json!("write"));
        fields.insert(
            event_keys::PRIMITIVE_SUBTYPE.to_string(),
            json!("staging_write"),
        );

        let event = Event {
            ts_ms: 1704067203000,
            host: "staging-compute-04".to_string(),
            tags: vec![
                "linux".to_string(),
                "exfiltration".to_string(),
                "ebpf".to_string(),
            ],
            proc_key: Some("host:staging-compute-04:pid:5897".to_string()),
            file_key: Some("host:staging-compute-04:path:/tmp/output_2024.data".to_string()),
            identity_key: Some("host:staging-compute-04:uid:1000".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "linux_capture_0".to_string(),
                segment_id: 43,
                record_index: 312,
            }),
            fields,
        };

        let json_out = event_to_json(&event);
        println!("=== CANONICAL: exfiltration (staging_write) ===");
        println!("{}", serde_json::to_string_pretty(&json_out).unwrap());

        assert_eq!(json_out["tags"][1], "exfiltration");
        assert_eq!(
            json_out["fields"][event_keys::PRIMITIVE_SUBTYPE],
            "staging_write"
        );
        assert_eq!(json_out["fields"][event_keys::FILE_OP], "write");
    }

    #[test]
    fn example_network_connection_canonical_json() {
        // Canonical network_connection event (external C&C)
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(6234));
        fields.insert(event_keys::PROC_UID.to_string(), json!(1000));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(1000));
        fields.insert(event_keys::NET_REMOTE_IP.to_string(), json!("203.0.113.92"));
        fields.insert(event_keys::NET_REMOTE_PORT.to_string(), json!(4444));
        fields.insert(event_keys::NET_SUSPICIOUS_PORT.to_string(), json!(true));

        let event = Event {
            ts_ms: 1704067204000,
            host: "production-worker-05".to_string(),
            tags: vec![
                "linux".to_string(),
                "network_connection".to_string(),
                "ebpf".to_string(),
            ],
            proc_key: Some("host:prod-worker-05:pid:6234".to_string()),
            file_key: None,
            identity_key: Some("host:prod-worker-05:uid:1000".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "linux_capture_0".to_string(),
                segment_id: 44,
                record_index: 89,
            }),
            fields,
        };

        let json_out = event_to_json(&event);
        println!("=== CANONICAL: network_connection ===");
        println!("{}", serde_json::to_string_pretty(&json_out).unwrap());

        assert_eq!(json_out["tags"][1], "network_connection");
        assert_eq!(json_out["fields"][event_keys::NET_SUSPICIOUS_PORT], true);
        assert_eq!(json_out["fields"][event_keys::NET_REMOTE_PORT], 4444);
    }

    #[test]
    fn example_macos_credential_access_canonical_json() {
        // macOS canonical credential_access event (security keychain)
        let mut fields = BTreeMap::new();
        fields.insert(event_keys::PROC_PID.to_string(), json!(1847));
        fields.insert(event_keys::PROC_UID.to_string(), json!(501));
        fields.insert(event_keys::PROC_EUID.to_string(), json!(501));
        fields.insert(event_keys::PROC_EXE.to_string(), json!("/usr/bin/security"));
        fields.insert(
            event_keys::PROC_ARGV.to_string(),
            json!(["security", "find-identity", "-v"]),
        );
        fields.insert(event_keys::CRED_TOOL.to_string(), json!("security"));

        let event = Event {
            ts_ms: 1704067300000,
            host: "mbp-jsmith".to_string(),
            tags: vec![
                "macos".to_string(),
                "credential_access".to_string(),
                "bsm".to_string(),
            ],
            proc_key: Some("host:mbp-jsmith:pid:1847".to_string()),
            file_key: None,
            identity_key: Some("host:mbp-jsmith:uid:501".to_string()),
            evidence_ptr: Some(EvidencePtr {
                stream_id: "macos_capture_0".to_string(),
                segment_id: 15,
                record_index: 402,
            }),
            fields,
        };

        let json_out = event_to_json(&event);
        println!("=== CANONICAL: macOS credential_access ===");
        println!("{}", serde_json::to_string_pretty(&json_out).unwrap());

        assert_eq!(json_out["tags"][0], "macos");
        assert_eq!(json_out["tags"][1], "credential_access");
    }

    #[test]
    fn schema_contract_summary() {
        // Document the locked schema contract for all canonical event types

        let contract = r#"
=== LOCKED SCHEMA CONTRACT ===

Event Structure (core/src/event.rs):
- ts_ms: i64              // Milliseconds since epoch
- host: String            // Hostname 
- tags: Vec<String>       // [platform, event_type, sensor]
- proc_key: Option<String>
- file_key: Option<String>
- identity_key: Option<String>
- evidence_ptr: Option<EvidencePtr> // Set ONLY by capture writer
- fields: BTreeMap<String, Value>   // All using event_keys:: constants

Canonical Event Types (tags[1]):
1. credential_access
2. discovery
3. exfiltration
4. network_connection

Required Fields by Type:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

credential_access:
- PROC_PID ("pid")            [u32]
- PROC_UID ("uid")            [u32]
- PROC_EUID ("euid")          [u32]
- PROC_EXE ("exe")            [String]
- CRED_TOOL ("cred_tool")     [String] - ssh, sudo, gpg, security, etc.
- PROC_ARGV ("argv")          [Array] - optional but recommended

discovery:
- PROC_PID ("pid")            [u32]
- PROC_UID ("uid")            [u32]
- PROC_EUID ("euid")          [u32]
- PROC_EXE ("exe")            [String]
- DISCOVERY_TOOL ("discovery_tool") [String] - whoami, id, ps, netstat, etc.
- PROC_ARGV ("argv")          [Array] - optional but recommended

exfiltration (CRITICAL: must have primitive_subtype):
  archive_tool_exec:
  - PROC_PID, PROC_UID, PROC_EUID, PROC_EXE
  - ARCHIVE_TOOL ("archive_tool")           [String] - tar, zip, gzip, etc.
  - PRIMITIVE_SUBTYPE ("primitive_subtype") [String] = "archive_tool_exec" (REQUIRED)
  - PROC_ARGV ("argv")                      [Array]
  
  staging_write:
  - PROC_PID, PROC_UID, PROC_EUID
  - FILE_PATH ("path")                      [String] - /tmp, ~/Downloads, ~/.cache, etc.
  - FILE_OP ("op")                          [String] - "write", "create"
  - PRIMITIVE_SUBTYPE ("primitive_subtype") [String] = "staging_write" (REQUIRED)
  - PROC_EXE ("exe")                        [String] - optional

network_connection:
- PROC_PID ("pid")            [u32]
- PROC_UID ("uid")            [u32]
- PROC_EUID ("euid")          [u32]
- NET_REMOTE_IP ("remote_ip") [String] - external IP only (no RFC1918)
- NET_REMOTE_PORT ("remote_port") [u16]
- NET_SUSPICIOUS_PORT ("suspicious_port") [bool] - optional

EvidencePtr Assignment:
- ONLY assigned by capture writer (macOS: write_segment_atomic, Linux: poll_and_write)
- Assigned AFTER sort and dedup to ensure record_index reflects final position
- segment_id matches actual segment sequence number (NOT hardcoded)

Deduplication:
- macOS: segment_derived_seen HashSet per segment
- Linux: seen_derived HashSet per poll
- Key: (ts_ms, pid, uid, event_type, type_specific_fields)
- Type-specific fields: remote_ip+port (network), path+op (staging), exe+tool (exec-derived)

Validation:
- Event::validate_canonical_primitive() enforces all schema requirements
- Called before playbook processing to catch violations early

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LOCKED: Do not emit events with:
  ❌ Non-canonical event_types (only 4 allowed)
  ❌ Hardcoded field names (all must use event_keys:: constants)
  ❌ Exfiltration without primitive_subtype
  ❌ Missing required fields
  ❌ EvidencePtr set before capture writer
        "#;

        println!("{}", contract);
    }
}
