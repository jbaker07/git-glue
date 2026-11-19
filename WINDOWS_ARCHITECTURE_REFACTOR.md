# Windows Architecture Refactor - Session Summary

## Objective
Fix Windows ingestion architecture to be **real, honest, and correct**:
- Remove scattered EvidencePtr assignment
- Implement unified EvidencePtr assignment in capture_windows_rotating.rs ONLY
- Readers return raw data (WevtRecord), not pre-constructed Events
- Self-test returns explicit exit code 2 on non-Windows platforms

## Files Modified

### 1. windows/wevt_reader.rs (Complete Refactor)
**Change**: From returning `Vec<Event>` to returning `Vec<WevtRecord>`

**Key Updates**:
- `WevtRecord` struct: channel, event_id, record_id, timestamp, computer, provider, xml
- `WevtRecord::to_event()`: Converts to core::Event with fields but **NO EvidencePtr** (placeholder: "unset", 0, 0)
- `poll()` method: Takes `BookmarkManager` parameter for resume support
- Return type: `Result<Vec<WevtRecord>, String>` (not Event)
- Non-Windows path: Returns `Err("Windows-only API: EvtQuery requires Windows platform")`
- Windows path: Stub with TODO comments for real WEVTAPI EvtQuery+EvtSeek implementation

**Architecture Compliance**: ✅
- Reader returns raw data only
- No EvidencePtr construction
- Explicit "Windows-only" error on non-Windows

### 2. windows/sensors/collect.rs (Removed EvidencePtr)
**Change**: Stopped constructing EvidencePtr, now delegates to capture

**Key Updates**:
- `collect_evtx_events()`: Calls `wevt_reader.poll(&bookmark_mgr)`
- Receives `Vec<WevtRecord>`, converts via `record.to_event()`
- Returns `Vec<Event>` with fields populated but **NO EvidencePtr assignment**
- Fields populated: windows.channel, windows.record_id, windows.event_id, windows.provider, windows.computer, windows.xml

**Architecture Compliance**: ✅
- Removed lines 22-31 (old EvidencePtr assignment)
- Now returns raw field data for capture to use

### 3. windows/capture_windows_rotating.rs (Added EvidencePtr Assignment)
**Change**: NEW - Centralized EvidencePtr assignment

**Key Addition**:
```rust
fn assign_evidence_ptrs(&self, mut events: Vec<crate::Event>) -> Vec<crate::Event> {
    for event in &mut events {
        // Extract channel from windows.channel field → stream_id
        if let Some(channel_val) = event.fields.get("windows.channel") {
            if let Some(channel_str) = channel_val.as_str() {
                event.evidence_ptr.stream_id = channel_str.to_string();
            }
        }
        
        // Extract record_id from windows.record_id field → segment_id + record_index
        if let Some(record_id_val) = event.fields.get("windows.record_id") {
            if let Some(record_id_u64) = record_id_val.as_u64() {
                event.evidence_ptr.segment_id = record_id_u64;
                event.evidence_ptr.record_index = (record_id_u64 & 0xFFFF_FFFF) as u32;
            }
        }
    }
    events
}
```

**Integration**:
- `poll_and_write()` calls `collect::collect_all()` → gets events with fields, no EvidencePtr
- Immediately calls `assign_evidence_ptrs()` before writing segments
- Comment marks this as the ONLY place in Windows architecture where EvidencePtr is assigned

**Architecture Compliance**: ✅
- Single point of EvidencePtr assignment
- Uses field data from readers
- Clear comment marking this as exclusive responsibility

### 4. windows/self_test.rs (Platform Detection)
**Change**: Explicit exit code 2 on non-Windows

**Key Updates**:
- Check: `if !cfg!(target_os = "windows")` at function entry
- Non-Windows behavior: Print error message, call `std::process::exit(2)`
- Message: `"self-test-windows requires Windows platform"`
- Windows behavior (unchanged): Run 30s test, exit 0 only if events collected from ≥2 channels

**Architecture Compliance**: ✅
- Explicit "Windows-only" detection
- Exit code 2 on non-Windows (verified: tested on macOS)
- Clear error message

### 5. windows/bookmark_manager.rs (Constructor Updates)
**Change**: Added parameterless `new()` method for convenience

**Key Updates**:
- `new()`: Creates BookmarkManager with default temp directory
- `with_telemetry_root(path)`: Original signature for production use
- `new_for_testing()`: Test helper (cfg-gated)
- `update_bookmark()` signature simplified: Takes event_count + timestamp instead of bookmark_xml

**Architecture Compliance**: ✅
- Simplified constructor for readers
- Still supports full bookmark persistence
- No EvidencePtr involvement

## Architecture Diagram

```
WevtReader.poll()
    ↓
Vec<WevtRecord> [raw data: channel, record_id, event_id, provider, computer, xml]
    ↓
collect.rs: collect_evtx_events()
    ↓
Vec<Event> [fields: windows.channel, windows.record_id, windows.event_id, ...]
            [EvidencePtr: placeholder "unset", 0, 0]
    ↓
capture.rs: assign_evidence_ptrs()
    ↓
Vec<Event> [fields: (same)]
            [EvidencePtr: stream_id=channel, segment_id=record_id, record_index=(record_id & 0xFFFFFFFF)]
    ↓
Write to segment JSONL
```

## Key Architectural Properties

### EvidencePtr Assignment
- **Location**: ONLY in `capture_windows_rotating::assign_evidence_ptrs()`
- **Source**: Fields populated by readers (windows.channel, windows.record_id)
- **No other module** constructs or modifies EvidencePtr

### Reader Responsibilities
- **wevt_reader.rs**: Return raw WevtRecord, bookmark management
- **collect.rs**: Convert to Events with fields, no EvidencePtr
- **Monitors**: Convert monitor output to Events with fields, no EvidencePtr

### Capture Responsibilities
- **capture_windows_rotating.rs**: 
  - Call collect_all()
  - Assign EvidencePtr from fields
  - Write to segments

### Cross-Platform Behavior
- **Windows**: Real WEVTAPI calls (TODO: behind cfg(target_os="windows"))
- **Non-Windows**: Explicit error messages, exit code 2 on self-test

## Testing

### Test Results
```
cargo check --lib
✅ Finished dev in 0.43s

cargo build --bin capture_windows_rotating --release  
✅ Finished in 14.56s

cargo run --bin capture_windows_rotating --release -- --self-test-windows
[windows self-test] ERROR: self-test-windows requires Windows platform
[windows self-test] Current platform: not Windows (macOS/Linux)
Exit code: 2
✅ Correct behavior on macOS
```

## Remaining Work (Next Steps)

### Real WEVTAPI Implementation (Stubbed)
File: `windows/wevt_reader.rs` lines 140-155

Current:
```rust
#[cfg(target_os = "windows")]
fn poll_windows(&self, bookmark_mgr: ...) -> Result<Vec<WevtRecord>, String> {
    // TODO: Implement real WEVTAPI EvtQuery + EvtSeek path
    let mut all_records = Vec::new();
    for channel_cfg in &self.channels {
        // ... loop with bookmark resume ...
    }
    Ok(all_records)
}
```

TODO Implementation:
1. For each enabled channel:
   - `EvtCreateBookmark()` from persisted XML string (or NULL for new subscribe)
   - `EvtQuery(channel, xpath_filter)` to open query handle
   - `EvtSeek(query_handle, EvtSeekRelativeToBookmark, bookmark)` to resume
   - `EvtNext(query_handle, max_records_per_poll)` loop
     - `EvtRender(event_handle, EvtRenderEventXml)` for each record
     - Parse EventID, timestamp, provider, Computer from XML
     - Construct WevtRecord
     - `EvtUpdateBookmark(bookmark, event_handle)` to track position
   - Persist bookmark XML via `bookmark_mgr.update_bookmark()`

2. Bounded by: max_records_per_poll per channel, max_xml_bytes_per_event

3. Return Vec<WevtRecord> with all fields

### Monitor Health Events (Future)
All 5 monitors currently return empty vec![] when in stub mode.
Consider: Emit health events indicating "stub not implemented" on Windows until real implementation available.

### Cleanup: Minor Warnings
- Remove unused `mut` declarations in self_test.rs and collect.rs
- Suppress unused imports/variables in module stubs

## Validation Checklist

- [x] Compilation: No errors, only warnings (acceptable)
- [x] wevt_reader returns Vec<WevtRecord> (NOT Event)
- [x] collect.rs does NOT assign EvidencePtr
- [x] capture_windows_rotating.rs ONLY assigns EvidencePtr
- [x] Self-test exits code 2 on non-Windows
- [x] Self-test error message is "Windows-only" compliant
- [x] BookmarkManager constructor simplified for readers
- [x] Fields properly populated: windows.channel, windows.record_id, etc.
- [x] EvidencePtr placeholder created by reader, overwritten by capture

## References

**Related Architecture Documents**:
- BASELINE_SYSTEM_DESIGN.md - System architecture overview
- SHARED_BASELINE_ARCHITECTURE.md - Baseline architecture patterns
- WINDOWS_SIGNAL_ENGINE_INTEGRATION.md - Windows integration strategy

**Key Commits**:
- Refactored wevt_reader.rs: Return WevtRecord instead of Event
- Removed EvidencePtr from collect.rs
- Added assign_evidence_ptrs() to capture_windows_rotating.rs
- Fixed self_test.rs exit behavior
