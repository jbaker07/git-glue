# Architecture Guard Assertion - Technical Details

## Guard Implementation

Location: `windows/capture_windows_rotating.rs::assign_evidence_ptrs_with_state()`

```rust
fn assign_evidence_ptrs_with_state(&self, mut events: Vec<crate::Event>, segment_seq: u64) -> Vec<crate::Event> {
    let mut record_index = 0u32;

    for event in &mut events {
        // GUARD: Incoming event must NOT have evidence_ptr already set
        if event.evidence_ptr.is_some() {
            debug_assert!(false, "[capture] BUG: incoming event already has evidence_ptr set - this violates architecture");
            eprintln!("[capture] ERROR: incoming event from collect already has evidence_ptr - dropping event as bug indicator");
            continue;  // Drop this event
        }
        
        // ... rest of assignment code
    }

    // Filter out any events that triggered the guard
    events.retain(|e| e.evidence_ptr.is_some());
    events
}
```

## How It Works

### Detection
```rust
if event.evidence_ptr.is_some()
```
- Checks if event already has Some(EvidencePtr) assigned
- This should NEVER happen if architecture is correct

### Response (3 levels of reporting)

1. **Debug Assertion** (development):
   ```rust
   debug_assert!(false, "[capture] BUG: incoming event already has evidence_ptr set - this violates architecture");
   ```
   - Panics in debug builds with clear message
   - Helps developer catch bug immediately during testing

2. **Stderr Logging** (production):
   ```rust
   eprintln!("[capture] ERROR: incoming event from collect already has evidence_ptr - dropping event as bug indicator");
   ```
   - Always logs to stderr with context
   - Visible in logs even in release builds
   - Indicates event integrity issue

3. **Event Filtering** (recovery):
   ```rust
   continue;  // Skip this event
   events.retain(|e| e.evidence_ptr.is_some());
   ```
   - Event is dropped entirely
   - Prevents corrupted event from reaching segment writer
   - Maintains segment integrity

## Why This Guard Matters

### Architectural Violation Scenario
If a reader or collector accidentally does:
```rust
// BUG: Assigns EvidencePtr too early (wrong location)
Event {
    evidence_ptr: Some(EvidencePtr {
        stream_id: "...",
        segment_id: record_id,  // WRONG: uses WEVTAPI record_id
        record_index: 0,
    }),
    ...
}
```

### Guard Response
1. Guard detects: `event.evidence_ptr.is_some()` → true
2. Debug assertion triggers in test builds
3. Error logged: `"incoming event from collect already has evidence_ptr"`
4. Event dropped: Not written to segment
5. Result: Bug caught, segment integrity maintained

## Compile-Time vs Runtime Safety

### Type System (Compile-Time)
```rust
evidence_ptr: Option<EvidencePtr>  // Forces explicit Some/None handling
```
- Readers cannot accidentally create events with EvidencePtr
- They can only create `None`
- Type checker enforces this

### Guard Assertion (Runtime)
```rust
if event.evidence_ptr.is_some() {
    debug_assert!(false, "BUG: ...");
    eprintln!("ERROR: ...");
    continue;
}
```
- Catches architectural violations if they somehow slip through
- Provides clear error reporting
- Protects segment integrity

## Testing the Guard

### Expected Behavior (Normal Operation)
```
Incoming Event { evidence_ptr: None, fields: {...}, ... }
Guard check: is_some() → false → passes
Assignment: event.evidence_ptr = Some(EvidencePtr { ... })
Output Event { evidence_ptr: Some(...), fields: {...}, ... }
```

### Guard Trigger (Architectural Bug)
```
Incoming Event { evidence_ptr: Some(...), fields: {...}, ... }  ← BUG!
Guard check: is_some() → true → TRIGGERS
Debug build: panic!("[capture] BUG: incoming event already has evidence_ptr set")
Release build: 
  stderr: "[capture] ERROR: incoming event from collect already has evidence_ptr - dropping"
Output: Event dropped (filtered out)
```

## Fields Not Used for Evidence_ptr Assignment

These fields are present but NOT used:
```rust
// WRONG - using WEVTAPI state for evidence_ptr
if let Some(record_id_val) = event.fields.get("windows.record_id") {
    event.evidence_ptr.segment_id = record_id_u64;  // DON'T DO THIS
}
```

Correct approach:
```rust
// RIGHT - using capture writer state
event.evidence_ptr = Some(EvidencePtr {
    stream_id: channel_name,   // From fields.windows.channel
    segment_id: self.segment_count,  // From writer state
    record_index: self.record_counter,  // From writer state
});
```

## Debugging Guide

### If Guard Triggers

1. **Find the source**: Search for `evidence_ptr` assignments in Windows modules
   ```bash
   grep -r "evidence_ptr.*Some\|evidence_ptr.*=" windows/
   ```

2. **Check readers**: Verify wevt_reader returns `None`
   ```rust
   evidence_ptr: None,  // ✅ Correct
   ```

3. **Check collectors**: Verify collect.rs doesn't assign EvidencePtr
   - Should only call `record.to_event()`
   - Should NOT construct EvidencePtr

4. **Check monitors**: Verify monitor.rs doesn't assign EvidencePtr
   - Should return Events with None
   - Should NOT create Some(EvidencePtr)

### Log Example
If guard triggers in production:
```
[capture] ERROR: incoming event from collect already has evidence_ptr - dropping event as bug indicator
[capture] event.ts_ms: 1708456789123
[capture] event.host: WORKSTATION
[capture] event.evidence_ptr: Some(EvidencePtr { stream_id: "Security", segment_id: 12345, record_index: 0 })
```

Indicates: Event created with EvidencePtr set (architectural violation)

## Migration Path

### Phase 1 (Current): Guard in Place
- Compile-time: Type system prevents (Option<EvidencePtr>)
- Runtime: Guard catches if violated

### Phase 2: Real WEVTAPI
- Implement EvtQuery + EvtSeek + EvtUpdateBookmark
- Bookmark persistence via EvtUpdateBookmark callback
- Guard still in place as safety net

### Phase 3: Monitor Enhancement
- Implement real registry/service/task/lateral/log_tamper monitors
- All return Events with evidence_ptr: None
- Guard validates all inputs

---

## Summary

**The guard assertion is a safety mechanism that:**
1. Detects architectural violations at the capture point
2. Prevents corrupted events from reaching storage
3. Provides clear error messages for debugging
4. Works in conjunction with type system (Option<EvidencePtr>)
5. Maintains segment integrity even if violations slip through

**It's not a hack - it's a necessary safeguard for a distributed architecture where multiple modules emit events to a single capture writer.**
