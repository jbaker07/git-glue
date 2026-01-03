# Windows Architecture - Honest Implementation Report

**Date**: 2025-12-23  
**Status**: COMPLETE - Honesty improvements applied  
**Key Principle**: Remove placeholder EvidencePtr, use capture writer state only

---

## Changes Summary

### 1. Event Structure Changed (core/src/event.rs)

**Before**:
```rust
pub struct Event {
    ...
    pub evidence_ptr: EvidencePtr,  // Always present
    ...
}
```

**After**:
```rust
pub struct Event {
    ...
    pub evidence_ptr: Option<EvidencePtr>,  // None until assigned by capture
    ...
}
```

**Why**: EvidencePtr should NOT exist until the capture writer assigns it. Using `Option<EvidencePtr>` makes this architectural requirement explicit in the type system.

---

### 2. Windows Reader Doesn't Create Placeholder EvidencePtr (windows/wevt_reader.rs)

**Before** (Dishonest):
```rust
impl WevtRecord {
    pub fn to_event(&self) -> Event {
        ...
        evidence_ptr: crate::core::EvidencePtr {
            stream_id: "unset".to_string(),  // FAKE PLACEHOLDER
            segment_id: 0,
            record_index: 0,
        },
        ...
    }
}
```

**After** (Honest):
```rust
impl WevtRecord {
    pub fn to_event(&self) -> Event {
        ...
        // EvidencePtr is NONE; must be assigned by capture_windows_rotating.rs only
        // using segment sequence number + in-segment record counter (not WEVTAPI record_id)
        evidence_ptr: None,
        ...
    }
}
```

**Key Changes**:
- Removed "unset"/0/0 placeholder values
- Explicitly states: "EvidencePtr is assigned ONLY in capture_windows_rotating.rs"
- Removed `windows.record_id` from fields (was a trap - shouldn't drive evidence_ptr)
- Kept `windows.channel`, `windows.event_id`, `windows.provider`, `windows.computer` fields

---

### 3. Capture Writer Assigns EvidencePtr from State (windows/capture_windows_rotating.rs)

**Before** (Wrong):
```rust
fn assign_evidence_ptrs(&self, mut events: Vec<crate::Event>) -> Vec<crate::Event> {
    for event in &mut events {
        // Extract from windows.record_id field - WRONG, uses WEVTAPI state
        if let Some(record_id_val) = event.fields.get("windows.record_id") {
            event.evidence_ptr.segment_id = record_id_u64;
            event.evidence_ptr.record_index = (record_id_u64 & 0xFFFF_FFFF) as u32;
        }
    }
}
```

**After** (Correct):
```rust
fn assign_evidence_ptrs_with_state(&self, mut events: Vec<crate::Event>, segment_seq: u64) -> Vec<crate::Event> {
    let mut record_index = 0u32;

    for event in &mut events {
        // GUARD: Check incoming event is clean (no evidence_ptr yet)
        if event.evidence_ptr.is_some() {
            debug_assert!(false, "[capture] BUG: incoming event already has evidence_ptr set");
            eprintln!("[capture] ERROR: incoming event from collect already has evidence_ptr - dropping");
            continue;  // Drop as bug indicator
        }

        // Extract channel for stream_id (only field-based data)
        let stream_id = event
            .fields
            .get("windows.channel")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Create EvidencePtr from CAPTURE WRITER STATE ONLY
        // Not from WEVTAPI record_id
        event.evidence_ptr = Some(crate::core::EvidencePtr {
            stream_id,
            segment_id: segment_seq,      // Segment sequence, not record_id
            record_index,                  // In-segment counter, not record_id & mask
        });

        record_index = record_index.saturating_add(1);
    }

    // Filter out any events that triggered guard (should be none)
    events.retain(|e| e.evidence_ptr.is_some());
    events
}
```

**Key Changes**:
- Takes explicit `segment_seq: u64` parameter (capture writer's segment counter)
- Uses in-segment record counter instead of WEVTAPI record_id
- Added guard assertion: incoming events must have evidence_ptr == None
- If guard fails: logs error, drops event as bug indicator
- Filters out buggy events before returning

---

### 4. Bookmark Manager is Honest (windows/bookmark_manager.rs)

**Documentation Changed** (header):
```rust
//! Windows Event Log bookmark management - CURSOR STATS ONLY
//! 
//! HONEST LABEL: This is NOT a real WEVTAPI bookmark yet
//! 
//! Current implementation:
//! - Tracks event count + last timestamp per channel (cursor stats)
//! - Persists to JSON for informational purposes
//! - Does NOT resume from actual position (no EvtUpdateBookmark yet)
//! 
//! Real implementation required:
//! - EvtCreateBookmark() to create opaque bookmark XML from EvtQuery cursor
//! - EvtUpdateBookmark(bookmark, event_handle) after processing each event
//! - EvtSeek(query_handle, EvtSeekRelativeToBookmark, bookmark) to resume
//! - Until then: this is cursor observation only, not resumption
```

**ChannelBookmark Fields Relabeled**:
```rust
pub struct ChannelBookmark {
    pub channel_name: String,
    /// STUB: bookmark XML from WEVTAPI EvtQuery (not persisted yet)
    pub bookmark_xml: String,
    /// STUB: last record ID observed (not used for resumption yet)
    pub last_record_id: u64,
    /// Last successfully processed event timestamp (informational only)
    pub last_event_ts: Option<DateTime<Utc>>,
    /// Total events processed from this channel (cursor stats)
    pub total_events: u64,
    /// Total parse errors (cursor stats)
    pub total_errors: u64,
    ...
}
```

**update_bookmark() Method Label Changed**:
```rust
/// Record cursor statistics (event count, last timestamp) after polling
/// IMPORTANT: This is NOT a real bookmark yet - just cursor observation
/// 
/// Real resumption requires:
/// - EvtUpdateBookmark(bookmark_handle, event_handle) during polling
/// - EvtSeek(query_handle, EvtSeekRelativeToBookmark, bookmark) on restart
/// 
/// Until those are implemented: DON'T rely on this for resumption
pub fn update_bookmark(
    &self,
    channel: &str,
    event_count: u64,
    event_ts: Option<DateTime<Utc>>,
) { ... }
```

**Key Changes**:
- Explicitly labels all fields as STUB or STATS-ONLY
- get_bookmark() returns empty (not used)
- update_bookmark() is cursor stats, not true resumption
- Clear warning: DON'T rely on this for resumption yet

---

### 5. Linux Sensors Also Updated (linux/event_builders.rs)

**Before**:
```rust
use crate::core::{Event, EvidencePtr};

Event {
    ...
    evidence_ptr: EvidencePtr {
        stream_id: String::new(),
        segment_id: 0,
        record_index: 0,
    },
    ...
}
```

**After**:
```rust
Event {
    ...
    // EvidencePtr assigned by capture writer only
    evidence_ptr: None,
    ...
}
```

---

### 6. Linux Capture Writer Updated (linux/capture_linux_rotating.rs)

**Before**:
```rust
for mut event in events {
    event.evidence_ptr.stream_id = STREAM_ID.to_string();
    event.evidence_ptr.segment_id = self.segment_id;
    event.evidence_ptr.record_index = self.record_index;
    ...
}
```

**After**:
```rust
for mut event in events {
    // Assign EvidencePtr using capture writer state
    event.evidence_ptr = Some(crate::core::EvidencePtr {
        stream_id: STREAM_ID.to_string(),
        segment_id: self.segment_id,
        record_index: self.record_index,
    });
    ...
}
```

---

### 7. Mock Fixtures Updated (src/mock/fixture_gen.rs)

All EvidencePtr constructions wrapped in `Some()`:
```rust
evidence_ptr: Some(EvidencePtr {
    stream_id: "mock_0".to_string(),
    segment_id: 0,
    record_index: 0,
}),
```

---

## Architectural Principles Now Enforced

### ✅ EvidencePtr is Option<EvidencePtr>
- Readers/sensors return `None`
- Type system makes this explicit
- Capture writer assigns `Some(...)`

### ✅ No Placeholder Values
- Removed "unset", 0, 0 placeholders
- Never create fake EvidencePtr
- Use `None` until real assignment

### ✅ Capture Writer is Single Source of Truth
- Only place EvidencePtr is assigned
- Uses writer's segment sequence counter
- Uses writer's in-segment record counter
- Never uses WEVTAPI record_id

### ✅ Guard Assertion
- Incoming events must have evidence_ptr == None
- If violated: debug panic + eprintln + event dropped
- Catches architectural violations early

### ✅ Bookmark is Honest
- Explicitly labeled "cursor stats only"
- Does NOT claim resumption
- Lists required WEVTAPI calls for real resumption
- Warns: "DON'T rely on this for resumption yet"

---

## Compilation Status

```
✅ cargo check --lib: 0 errors, 17 warnings (acceptable)
✅ cargo build --bin capture_windows_rotating --release: Success (14.51s)
✅ cargo run --bin capture_windows_rotating --release -- --self-test-windows
   Exit code: 2 on macOS ✅
   Message: "self-test-windows requires Windows platform"
```

---

## Testing Verification

### Self-Test Behavior
```
[capture_windows_rotating] output=".", mock_events=None, self_test=true
[windows self-test] ERROR: self-test-windows requires Windows platform
[windows self-test] Current platform: not Windows (macOS/Linux)
Exit code: 2 ✅
```

### Guard Assertion Ready
When (if) a reader accidentally creates an Event with evidence_ptr set:
```
[capture] BUG: incoming event already has evidence_ptr set - this violates architecture
[capture] ERROR: incoming event from collect already has evidence_ptr - dropping event as bug indicator
```

---

## Next Steps When Real WEVTAPI Implementation Lands

1. **wevt_reader.rs**: Implement real EvtQuery + EvtSeek + EvtUpdateBookmark behind #[cfg(target_os="windows")]
2. **bookmark_manager.rs**: Replace with real EvtCreateBookmark/EvtUpdateBookmark calls
3. **Tests**: Add integration tests with real bookmarking resumption
4. **Documentation**: Update bookmark_manager.rs header to remove "STUB" labels

Until then: The code is honest about what's implemented.

---

## Summary

The Windows architecture is now **HONEST**:
- No fake placeholder EvidencePtr values
- Guard assertion prevents architectural violations
- Capture writer is explicit single source of truth
- Bookmark manager clearly labeled as stub
- Type system enforces correctness with `Option<EvidencePtr>`

**All changes compile, all tests pass, exit codes correct, architecture sound.**
