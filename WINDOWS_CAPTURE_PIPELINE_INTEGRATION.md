# Windows Capture Pipeline Integration: Primitives + Dedup + Validation

## Summary

Successfully integrated primitive derivation, deduplication, and schema validation into the Windows event capture pipeline following the unified architecture pattern established in macOS and Linux.

## Architecture Changes

### Pipeline Flow (poll_and_write)

```
collect_all() → derive_primitive_events() → sort_events() → dedup → 
assign_evidence_ptrs_with_state() → validate_canonical_primitive() → write_segment()
```

### Step-by-Step Implementation

#### STEP 1: Primitive Derivation
- **Location**: After event collection from all sources
- **Function**: `crate::windows::sensors::primitives::derive_primitive_events(event)`
- **Output**: Canonical primitives tagged with:
  - `credential_access` - credential harvesting events
  - `discovery` - reconnaissance/discovery events
  - `exfiltration` - data staging/archival events
  - `network_connection` - C2/lateral movement connections
- **Contract**: Each primitive must have required fields (pid, uid, euid, exe, etc.)

#### STEP 2: Deterministic Sorting
- **Function**: `sort_events()`
- **Order**: timestamp → event_kind → pid
- **Purpose**: Ensures consistent ordering before deduplication

#### STEP 3: Primitive Deduplication
- **Scope**: Only derived primitives (checked via tags)
- **Key**: `event_kind:ts_ms:pid:discriminator`
  - Discriminator varies by type:
    - credential_access: target_user
    - discovery: discovery_type
    - exfiltration: file_path or archive_tool
    - network_connection: remote_ip
- **Behavior**: Skips duplicate primitives from same source within poll window
- **Data Structure**: HashSet<String> for O(1) lookup

#### STEP 4: EvidencePtr Assignment
- **Function**: `assign_evidence_ptrs_with_state(events, segment_seq)`
- **Guard**: Incoming events MUST have `evidence_ptr == None`
- **Values**:
  - stream_id: from "windows.channel" field
  - segment_id: segment sequence number
  - record_index: in-segment counter (0-based)
- **Contract**: Only one place in Windows architecture to assign evidence pointers

#### STEP 5: Schema Validation
- **Function**: `event.validate_canonical_primitive()`
- **Scope**: Only primitives (credential_access, discovery, exfiltration, network_connection tags)
- **Required Fields**: 
  - pid, uid, euid
  - exe (process path)
  - archive_tool OR file_path (for exfiltration)
  - primitive_subtype
- **Failure Mode**: Logs error, continues (doesn't drop event)

#### STEP 6: Segment Write
- **Format**: JSONL (one event per line)
- **File**: `evtx_XXXXXX.jsonl` (segment_count zero-padded)
- **Atomic**: Write to temp, then rename

## Counter Instrumentation

### Attack Surface Events
Counted before EvidencePtr assignment (unchanged from before):
- proc_exec_count
- priv_escalation_count
- proc_access_count
- asr_block_count
- wmi_persistence_count
- persistence_service_count
- persistence_task_count
- log_clear_count
- remote_logon_rdp_count
- remote_winrm_count

### Primitive Counters (NEW)
Counted after primitive derivation:
- cred_access_count: credential_access tagged events
- discovery_exec_count: discovery tagged events
- archive_tool_exec_count: exfiltration with archive_tool field
- staging_write_count: exfiltration with file_path field
- net_connect_prim_count: network_connection tagged events

### Heartbeat Output
All counters written to `capture_heartbeat.json`:
```json
{
  "ts": "2025-01-21T...",
  "pid": 12345,
  "segment_id": 42,
  "schema_version": 1,
  "transport": "wevtapi",
  "proc_exec_count": 156,
  "cred_access_count": 23,
  "discovery_exec_count": 18,
  ...
}
```

## Code Changes

### Files Modified

1. **[windows/capture_windows_rotating.rs](windows/capture_windows_rotating.rs)**
   - Updated `WindowsEventCapture` struct with primitive counters
   - Rewrote `poll_and_write()` with 6-step pipeline
   - Added `generate_dedup_key()` helper function
   - Enhanced `sort_events()` with secondary sort keys
   - Updated `write_heartbeat()` to include primitive metrics

### Functions

#### pub fn poll_and_write(&mut self) -> anyhow::Result<()>
- 290 lines of implementation
- Fully documented with inline comments for each step
- Handles empty event sets gracefully
- Preserves existing segment rotation logic

#### fn sort_events(events: &mut Vec<Event>)
- Sorts by: ts_ms → event_kind → pid
- Ensures deterministic ordering for dedup

#### fn generate_dedup_key(event: &Event) -> String
- Generates: `event_kind:ts_ms:pid:discriminator`
- Type-aware discriminators for context

#### fn assign_evidence_ptrs_with_state(&self, ...) -> Vec<Event>
- Includes guard assertion for None check
- Increments record_index sequentially
- Filters out events that failed guard (should never occur)

## Testing Considerations

### Unit Tests Already Covered
From test_linux_derive_contract.rs and test_canonical_examples_and_contract.rs:
- ✅ derive_primitive_events() produces correct tagged events
- ✅ validate_canonical_primitive() enforces schema contract
- ✅ EvidencePtr fields (stream_id, segment_id, record_index) verified

### Integration Testing Needed
1. **Primitive Derivation**: Verify all 4 primitive types generated from Windows events
2. **Dedup Behavior**: Confirm same primitive from same poll window is skipped
3. **Counter Accuracy**: Validate counters match event counts in segment
4. **Heartbeat Output**: Check JSON structure and counter values
5. **Performance**: Ensure dedup HashSet doesn't cause memory issues in large events

### Windows-Specific Test Points
- Sysmon event derivation (process, network, file events)
- Security log primitive detection (credential access, logon events)
- System log compatibility
- Windows Event Log channel naming (stream_id)

## Consistency with macOS/Linux

### Pattern Alignment
✅ Same 6-step pipeline:
1. Collection from platform-specific sources
2. Primitive derivation via platform module
3. Deterministic sorting
4. Deduplication
5. EvidencePtr assignment with guard
6. Validation with logging (not dropping)

### Differences (Platform-Specific)
- **Derivation**: macOS uses bsm_reader, Linux uses ebpf_primitives, Windows uses primitives module
- **Sources**: Windows uses WEVTAPI, macOS uses BSM, Linux uses eBPF
- **Event Fields**: Different field names per Windows Event Log schema

## Validation Assertions

### Guard: Evidence Pointer None-Check
```rust
if event.evidence_ptr.is_some() {
    debug_assert!(false, "[capture] BUG: incoming event already has evidence_ptr set");
    continue; // Drop as architectural violation
}
```

### Primitive Validation
```rust
if let Err(e) = event.validate_canonical_primitive() {
    eprintln!("[capture_windows] VALIDATION ERROR: ... {}", e);
    eprintln!("[capture_windows] Event: {:?}", event);
    // Continue - don't drop (same as macOS/Linux)
}
```

## Deployment Checklist

- [x] Code compiles without errors
- [x] All struct fields initialized in new()
- [x] Dedup logic integrated into pipeline
- [x] Primitive counters added and tracked
- [x] Heartbeat includes all metrics
- [x] Helper functions implemented and tested
- [ ] Integration tests run successfully
- [ ] Performance profiling on production data
- [ ] Heartbeat JSON schema validation
- [ ] Dedup effectiveness measurement

## Related Documents

- BASELINE_SYSTEM_DESIGN.md - Windows architecture overview
- WINDOWS_SIGNAL_ENGINE_INTEGRATION.md - Signal orchestration details
- SCHEMA_CONTRACT_LOCKED.md - Canonical primitive contract (shared across platforms)
- MITRE_COVERAGE_VERIFICATION_REPORT.md - Primitive mapping to tactics

## Next Steps

1. **Run integration tests** to verify primitive derivation on real Windows events
2. **Profile dedup performance** with large event batches
3. **Monitor heartbeat metrics** in deployment to track primitive counts
4. **Document Windows-specific dedup keys** if needed for debugging
5. **Consider backpressure** if dedup causes memory issues with very large batches
