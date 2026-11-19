# Windows Capture Pipeline: Code Structure Reference

## Function Call Hierarchy

```
WindowsEventCapture::poll_and_write()
  ├─ crate::windows::sensors::collect::collect_all(&host)
  │   └─ Returns: Vec<Event> with evidence_ptr = None
  │
  ├─ crate::windows::sensors::primitives::derive_primitive_events(event)
  │   └─ Returns: Vec<Event> with tags [credential_access|discovery|exfiltration|network_connection]
  │
  ├─ sort_events(&mut Vec<Event>)
  │   └─ Sorts by: ts_ms → event_kind → pid
  │
  ├─ generate_dedup_key(&Event) → String
  │   └─ Returns: "event_kind:ts_ms:pid:discriminator"
  │
  ├─ HashSet<String>::contains() / insert()
  │   └─ Dedup check O(1)
  │
  ├─ Counter accumulation
  │   ├─ Count attack surface events (proc_exec, priv_escalation, ...)
  │   └─ Count primitive events (cred_access, discovery, archive_tool, staging_write, net_connect)
  │
  ├─ assign_evidence_ptrs_with_state(&mut Vec<Event>, u64) → Vec<Event>
  │   ├─ Guard: assert event.evidence_ptr == None
  │   ├─ Extract stream_id from "windows.channel"
  │   ├─ Set segment_id from parameter
  │   └─ Set record_index (0-based counter)
  │
  ├─ event.validate_canonical_primitive() → Result<(), String>
  │   └─ Checks: pid, uid, euid, exe, archive_tool|file_path, primitive_subtype
  │
  ├─ fs::File::create() and writeln!() [JSONL]
  │   └─ Writes segment file
  │
  └─ write_heartbeat()
      └─ Writes JSON with all counters to capture_heartbeat.json
```

## Data Structures

### Event Flow State

```
Event (from collect_all)
├── ts_ms: u64
├── fields: BTreeMap<String, Value>
├── tags: Vec<String> [empty initially]
└── evidence_ptr: Option<EvidencePtr> = None

    ↓ (after derive_primitive_events)

Event (primitive)
├── ts_ms: u64
├── fields: BTreeMap<String, Value>
│   ├── "event_kind": "proc_exec" | "network_connect" | ...
│   ├── "pid": u64
│   ├── "uid": u32
│   ├── "euid": u32
│   ├── "exe": String
│   ├── "windows.channel": "Microsoft-Windows-Sysmon/Operational" | ...
│   └── [type-specific fields]
├── tags: Vec<String> = ["credential_access" | "discovery" | "exfiltration" | "network_connection"]
└── evidence_ptr: Option<EvidencePtr> = None

    ↓ (after assign_evidence_ptrs_with_state)

Event (complete)
├── ts_ms: u64
├── fields: BTreeMap<String, Value>
├── tags: Vec<String>
└── evidence_ptr: Some(EvidencePtr {
    stream_id: "Microsoft-Windows-Sysmon/Operational",
    segment_id: 42,
    record_index: 5
})
```

### Dedup Key Generation

```
Credential Access:
  key = "credential_access:1642778538123:4521:CONTOSO\admin"
                                 ↑       ↑    ↑
                              ts_ms     pid   target_user

Discovery:
  key = "net_enum:1642778538125:4521:domain_users"
                            ↑      ↑    ↑
                         ts_ms     pid   discovery_type

Exfiltration:
  key = "file_read:1642778538127:4521:C:\ProgramData\staging\data.7z"
                              ↑      ↑    ↑
                           ts_ms     pid   file_path OR archive_tool

Network Connection:
  key = "socket_connect:1642778538129:4521:192.168.1.100"
                                   ↑      ↑    ↑
                                ts_ms     pid   remote_ip
```

## Type-Aware Discriminators

```
┌─────────────────────────────────────────────────────────────────┐
│ Credential Access Primitives                                    │
├─────────────────────────────────────────────────────────────────┤
│ Fields required:                                                │
│ - target_user: String (discriminator for dedup)                │
│ - target_object: String (registry hive, process, file)        │
│ - access_type: String (read, write, execute)                  │
│ Example: Keylogging, SAM read, LSA dump                        │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Discovery Primitives                                            │
├─────────────────────────────────────────────────────────────────┤
│ Fields required:                                                │
│ - discovery_type: String (discriminator for dedup)            │
│   └─ Values: "domain_users", "domain_groups", "domain_admins" │
│              "network_shares", "local_users", "registry_enum"  │
│ Example: whoami, net user, Get-LocalGroup                      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Exfiltration Primitives                                         │
├─────────────────────────────────────────────────────────────────┤
│ Fields required (one of):                                       │
│ - archive_tool: String (7z.exe, rar.exe, etc.) - discriminator │
│ - file_path: String (staging location) - discriminator         │
│ - staging_size: u64 (bytes written)                           │
│ Example: RAR on staging drive, tar to /tmp                     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Network Connection Primitives                                   │
├─────────────────────────────────────────────────────────────────┤
│ Fields required:                                                │
│ - remote_ip: String (discriminator for dedup)                 │
│ - remote_port: u16                                             │
│ - protocol: String ("tcp", "udp")                             │
│ - connection_state: String ("established", "syn_sent", etc.)  │
│ Example: C2 beacon, lateral movement, DNS tunneling           │
└─────────────────────────────────────────────────────────────────┘
```

## Counter State Management

### WindowsEventCapture Struct Fields

```rust
pub struct WindowsEventCapture {
    // Configuration
    config: WindowsCaptureConfig,
    
    // Segment state
    segment_count: u64,        // Current segment number (monotonic)
    event_count: u64,          // Events in current segment
    
    // Data sources
    collector: crate::windows::sensors::EvtxCollector,
    metrics: BTreeMap<String, HeartbeatMetrics>,
    
    // ===== Attack Surface Event Counters =====
    // These count BASE events before derivation
    proc_exec_count: u64,           // Process execution
    priv_escalation_count: u64,     // Privilege escalation
    proc_access_count: u64,         // Process access/injection
    asr_block_count: u64,           // Attack Surface Reduction blocks
    wmi_persistence_count: u64,     // WMI event subscriptions
    persistence_service_count: u64, // Service installation/modification
    persistence_task_count: u64,    // Task scheduler persistence
    log_clear_count: u64,           // Event log clearing
    remote_logon_rdp_count: u64,    // RDP logon events
    remote_winrm_count: u64,        // WinRM remote execution
    decode_failed_count: u64,       // Event parsing failures
    
    // ===== Primitive Counters =====
    // These count DERIVED primitives after derive_primitive_events()
    cred_access_count: u64,         // Credential access primitives
    discovery_exec_count: u64,      // Discovery/enumeration primitives
    archive_tool_exec_count: u64,   // Archive utility executions (7z, rar, etc.)
    staging_write_count: u64,       // Suspicious file writes (exfiltration staging)
    net_connect_prim_count: u64,    // Network connection primitives (C2, lateral move)
}
```

### Counter Increments by Event Type

```
Base Event "proc_exec"
  ├─ proc_exec_count += 1          [attack surface]
  └─ If has exfiltration tag:
     └─ archive_tool_exec_count += 1  [primitive]
     OR staging_write_count += 1       [primitive]

Base Event "cred_access" (from SAM, LSA, etc.)
  ├─ [not in attack surface list]
  └─ cred_access_count += 1        [primitive]

Base Event "network_connect"
  ├─ [not in attack surface list]
  └─ net_connect_prim_count += 1   [primitive]

Derived Primitive with "discovery" tag
  └─ discovery_exec_count += 1     [primitive]
```

## Write Path

### Segment File

```
File: evtx_XXXXXX.jsonl (segment_count zero-padded)
Location: {config.root_dir}/evtx_000042.jsonl

Content (JSONL format):
{"ts_ms":1642778538123,"fields":{...},"tags":["proc_exec"],"evidence_ptr":{"stream_id":"...","segment_id":42,"record_index":0}}
{"ts_ms":1642778538125,"fields":{...},"tags":["credential_access"],"evidence_ptr":{"stream_id":"...","segment_id":42,"record_index":1}}
{"ts_ms":1642778538127,"fields":{...},"tags":["exfiltration"],"evidence_ptr":{"stream_id":"...","segment_id":42,"record_index":2}}
...
```

**Ordering**: Deterministic (ts_ms → event_kind → pid) per `sort_events()`

**Dedup**: Applied before writing (duplicates skipped)

### Heartbeat File

```
File: capture_heartbeat.json
Location: {config.root_dir}/capture_heartbeat.json
Frequency: Updated every 100ms via main loop

Content (JSON):
{
  "ts": "2025-01-21T14:32:18.123456Z",
  "pid": 5840,
  "segment_id": 42,
  "schema_version": 1,
  "transport": "wevtapi",
  
  "proc_exec_count": 156,
  "priv_escalation_count": 2,
  "proc_access_count": 89,
  "asr_block_count": 5,
  "wmi_persistence_count": 0,
  "persistence_service_count": 0,
  "persistence_task_count": 1,
  "log_clear_count": 0,
  "remote_logon_rdp_count": 23,
  "remote_winrm_count": 0,
  "decode_failed_count": 0,
  
  "cred_access_count": 12,
  "discovery_exec_count": 8,
  "archive_tool_exec_count": 2,
  "staging_write_count": 4,
  "net_connect_prim_count": 18
}
```

**Write Method**: Atomic (write to temp file, then rename)

## Guard Assertions

### EvidencePtr None Check

```
Location: assign_evidence_ptrs_with_state()
Trigger: event.evidence_ptr.is_some()

Action:
├─ debug_assert!(false, "[capture] BUG: incoming event has evidence_ptr set")
├─ continue (skip event)
└─ Filter out after loop: events.retain(|e| e.evidence_ptr.is_some())

Result: Event dropped as architectural violation
Purpose: Prevent double-assignment of EvidencePtr
```

### Validation Error Logging

```
Location: After EvidencePtr assignment
Trigger: event.validate_canonical_primitive() returns Err

Action:
├─ eprintln!("[capture_windows] VALIDATION ERROR: ...")
├─ eprintln!("[capture_windows] Event: {:?}", event)
└─ Continue (event retained, NOT dropped)

Result: Error logged, event written to segment
Purpose: Monitor schema compliance without losing data
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| derive_primitive_events() | O(n) per event | Platform-specific, bounded work |
| sort_events() | O(n log n) | On all events (base + derived) |
| dedup lookup | O(1) | HashSet contains check |
| dedup insert | O(1) | HashSet insert |
| Dedup total | O(n) | For all events |
| EvidencePtr assign | O(n) | Single pass with counter |
| Validation | O(n) | Single pass, checks per primitive |
| Write JSONL | O(n) | Single pass, serialization per event |
| **Total per poll** | **O(n log n)** | Dominated by sort |

**Memory**: O(n) for HashSet (dedup keys are short strings)

**Scaling**: Linear with event count (no quadratic operations)

## Integration Points

### Dependencies

```
windows/capture_windows_rotating.rs
  ├─ crate::windows::sensors::collect::collect_all()
  ├─ crate::windows::sensors::primitives::derive_primitive_events()
  ├─ crate::core::EvidencePtr
  ├─ crate::core::Event
  └─ crate::core::event_keys::*
```

### Called From

```
main() / server startup
  └─ WindowsEventCapture::run()
      └─ loop {
          ├─ poll_and_write()
          ├─ write_heartbeat()
          └─ sleep(100ms)
      }
```

## Testing Hooks

### Unit Test Patterns

```rust
#[test]
fn test_derive_windows_primitives() {
    // Use derive_primitive_events() from windows::sensors::primitives
    let base = create_base_process_event();
    let primitives = derive_primitive_events(&base);
    
    // Verify primitive tags
    assert!(primitives.iter().any(|e| e.tags.contains(&"credential_access".to_string())));
}

#[test]
fn test_dedup_key_generation() {
    let event = create_event_with_tags(vec!["credential_access"]);
    let key = generate_dedup_key(&event);
    
    // Key format: event_kind:ts_ms:pid:discriminator
    assert!(key.contains(":"));
    assert!(key.starts_with("credential_access:"));
}

#[test]
fn test_evidence_ptr_guard() {
    let mut event = create_event();
    event.evidence_ptr = Some(EvidencePtr { ... });
    
    // Guard should detect and skip this event
    let capture = WindowsEventCapture::new(temp_dir);
    let result = capture.assign_evidence_ptrs_with_state(vec![event], 0);
    
    // Event should be filtered out
    assert!(result.is_empty());
}
```

### Integration Test Patterns

```rust
#[test]
fn test_poll_and_write_integration() {
    let temp_dir = create_temp_dir();
    let mut capture = WindowsEventCapture::new(temp_dir.clone());
    
    // Mock collect_all() to return test events
    let result = capture.poll_and_write();
    
    assert!(result.is_ok());
    
    // Verify segment file exists
    let segment_path = temp_dir.join("evtx_000000.jsonl");
    assert!(segment_path.exists());
    
    // Verify heartbeat
    let heartbeat_path = temp_dir.join("capture_heartbeat.json");
    assert!(heartbeat_path.exists());
    let heartbeat: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&heartbeat_path)?)?;
    assert_eq!(heartbeat["schema_version"], 1);
    assert!(heartbeat.get("cred_access_count").is_some());
}
```

---

## Summary

The Windows capture pipeline is a complete, well-structured implementation of primitive derivation with deduplication and validation. All functions follow the established patterns from macOS and Linux, ensuring consistency across the platform-agnostic signal collection architecture.
