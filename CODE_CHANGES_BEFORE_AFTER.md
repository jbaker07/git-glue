# Code Changes - Before and After

## 1. Core Event Structure

### Before (core/src/event.rs)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts_ms: i64,
    pub host: String,
    pub tags: Vec<String>,
    pub proc_key: Option<String>,
    pub file_key: Option<String>,
    pub identity_key: Option<String>,
    /// Evidence pointer to the raw segment record that produced this Event
    pub evidence_ptr: EvidencePtr,  // ❌ Always present, can be fake
    pub fields: BTreeMap<String, serde_json::Value>,
}
```

### After (core/src/event.rs)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts_ms: i64,
    pub host: String,
    pub tags: Vec<String>,
    pub proc_key: Option<String>,
    pub file_key: Option<String>,
    pub identity_key: Option<String>,
    /// Evidence pointer to the raw segment record that produced this Event
    /// None until assignment by capture writer (segment seq + record counter)
    pub evidence_ptr: Option<EvidencePtr>,  // ✅ Optional, type system enforces
    pub fields: BTreeMap<String, serde_json::Value>,
}
```

---

## 2. Windows Reader

### Before (windows/wevt_reader.rs)
```rust
impl WevtRecord {
    /// Convert to core::Event with fields but NO EvidencePtr
    /// EvidencePtr is assigned ONLY in capture_windows_rotating.rs
    pub fn to_event(&self) -> Event {
        let mut fields = BTreeMap::new();

        // Store raw fields that capture will use to assign EvidencePtr
        fields.insert("windows.channel".to_string(), serde_json::json!(self.channel));
        fields.insert("windows.record_id".to_string(), serde_json::json!(self.record_id));  // ❌ TRAP
        fields.insert("windows.event_id".to_string(), serde_json::json!(self.event_id));
        // ...

        Event {
            ts_ms: self.timestamp.timestamp_millis(),
            host: self.computer.clone(),
            proc_key: None,
            file_key: None,
            identity_key: None,
            // EvidencePtr is a placeholder; will be overwritten by capture_windows_rotating.rs
            evidence_ptr: crate::core::EvidencePtr {
                stream_id: "unset".to_string(),  // ❌ FAKE
                segment_id: 0,                    // ❌ FAKE
                record_index: 0,                  // ❌ FAKE
            },
            fields,
            tags: vec![
                "windows".to_string(),
                "event_log".to_string(),
                self.channel.to_lowercase(),
            ],
        }
    }
}
```

### After (windows/wevt_reader.rs)
```rust
impl WevtRecord {
    /// Convert to core::Event with fields but NO EvidencePtr
    /// EvidencePtr is assigned ONLY in capture_windows_rotating.rs (via segment seq + record counter)
    pub fn to_event(&self) -> Event {
        let mut fields = BTreeMap::new();

        // Store raw fields needed by readers/analyzers
        // DO NOT include record_id in fields - evidence_ptr must come from capture writer state only
        fields.insert("windows.channel".to_string(), serde_json::json!(self.channel));
        fields.insert("windows.event_id".to_string(), serde_json::json!(self.event_id));
        fields.insert("windows.provider".to_string(), serde_json::json!(self.provider));
        fields.insert("windows.computer".to_string(), serde_json::json!(self.computer));
        // ... no record_id

        Event {
            ts_ms: self.timestamp.timestamp_millis(),
            host: self.computer.clone(),
            proc_key: None,
            file_key: None,
            identity_key: None,
            // EvidencePtr is NONE; must be assigned by capture_windows_rotating.rs only
            // using segment sequence number + in-segment record counter (not WEVTAPI record_id)
            evidence_ptr: None,  // ✅ HONEST
            fields,
            tags: vec![
                "windows".to_string(),
                "event_log".to_string(),
                self.channel.to_lowercase(),
            ],
        }
    }
}
```

---

## 3. Windows Capture Writer

### Before (windows/capture_windows_rotating.rs)
```rust
/// Assign EvidencePtr to events using fields set by readers
/// stream_id comes from windows.channel
/// segment_id comes from windows.record_id
fn assign_evidence_ptrs(&self, mut events: Vec<crate::Event>) -> Vec<crate::Event> {
    for event in &mut events {
        // Extract channel from fields
        if let Some(channel_val) = event.fields.get("windows.channel") {
            if let Some(channel_str) = channel_val.as_str() {
                event.evidence_ptr.stream_id = channel_str.to_string();  // ❌ Mutating Option
            }
        }

        // Extract record_id from fields for segment_id and record_index
        if let Some(record_id_val) = event.fields.get("windows.record_id") {  // ❌ WEVTAPI state
            if let Some(record_id_u64) = record_id_val.as_u64() {
                event.evidence_ptr.segment_id = record_id_u64;  // ❌ Wrong state source
                event.evidence_ptr.record_index = (record_id_u64 & 0xFFFF_FFFF) as u32;  // ❌ Wrong source
            }
        }
    }
    events
}
```

### After (windows/capture_windows_rotating.rs)
```rust
/// Assign EvidencePtr to events using capture writer state (segment seq + record counter)
/// NEVER uses WEVTAPI record_id - that's internal Windows state not for evidence paths
/// 
/// Guard assertion: incoming events must have evidence_ptr == None
/// If any event already has Some(evidence_ptr), treat as architectural bug
fn assign_evidence_ptrs_with_state(&self, mut events: Vec<crate::Event>, segment_seq: u64) -> Vec<crate::Event> {
    let mut record_index = 0u32;

    for event in &mut events {
        // GUARD: Incoming event must NOT have evidence_ptr already set
        if event.evidence_ptr.is_some() {  // ✅ GUARD CHECK
            debug_assert!(false, "[capture] BUG: incoming event already has evidence_ptr set - this violates architecture");
            eprintln!("[capture] ERROR: incoming event from collect already has evidence_ptr - dropping event as bug indicator");
            // Drop this event - it's a sign of architectural violation
            continue;
        }

        // Extract channel for stream_id
        let stream_id = event
            .fields
            .get("windows.channel")  // ✅ Field only, no record_id
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Create EvidencePtr from capture writer state only
        event.evidence_ptr = Some(crate::core::EvidencePtr {  // ✅ Create Option::Some
            stream_id,
            segment_id: segment_seq,      // ✅ From writer state
            record_index,                  // ✅ From writer counter
        });

        record_index = record_index.saturating_add(1);
    }

    // Filter out any events that triggered the guard (should be none in normal operation)
    events.retain(|e| e.evidence_ptr.is_some());  // ✅ Guard protection

    events
}
```

---

## 4. Linux Event Builder

### Before (linux/event_builders.rs)
```rust
use crate::core::{Event, EvidencePtr};

pub fn event(
    host_ctx: &HostCtx,
    sensor_name: &str,
    kind: &str,
    severity: &str,
    mut fields: BTreeMap<String, Value>,
) -> Event {
    // ... field setup ...
    
    Event {
        ts_ms: host_ctx.now_ms(),
        host: host_ctx.hostname.clone(),
        tags: vec![
            "linux".to_string(),
            kind.to_string(),
            sensor_name.to_string(),
        ],
        proc_key: None,
        file_key: None,
        identity_key: None,
        evidence_ptr: EvidencePtr {  // ❌ Fake placeholder
            stream_id: String::new(),
            segment_id: 0,
            record_index: 0,
        },
        fields,
    }
}
```

### After (linux/event_builders.rs)
```rust
use crate::core::Event;

pub fn event(
    host_ctx: &HostCtx,
    sensor_name: &str,
    kind: &str,
    severity: &str,
    mut fields: BTreeMap<String, Value>,
) -> Event {
    // ... field setup ...
    
    Event {
        ts_ms: host_ctx.now_ms(),
        host: host_ctx.hostname.clone(),
        tags: vec![
            "linux".to_string(),
            kind.to_string(),
            sensor_name.to_string(),
        ],
        proc_key: None,
        file_key: None,
        identity_key: None,
        // EvidencePtr assigned by capture writer only
        evidence_ptr: None,  // ✅ HONEST
        fields,
    }
}
```

---

## 5. Linux Capture Writer

### Before (linux/capture_linux_rotating.rs)
```rust
for mut event in events {
    event.evidence_ptr.stream_id = STREAM_ID.to_string();  // ❌ Mutating struct fields
    event.evidence_ptr.segment_id = self.segment_id;
    event.evidence_ptr.record_index = self.record_index;
    
    self.write_event(&event)?;
    self.record_index += 1;
}
```

### After (linux/capture_linux_rotating.rs)
```rust
for mut event in events {
    // Assign EvidencePtr using capture writer state
    event.evidence_ptr = Some(crate::core::EvidencePtr {  // ✅ Create Option::Some
        stream_id: STREAM_ID.to_string(),
        segment_id: self.segment_id,
        record_index: self.record_index,
    });
    
    self.write_event(&event)?;
    self.record_index += 1;
}
```

---

## 6. Bookmark Manager Documentation

### Before (windows/bookmark_manager.rs)
```rust
//! Windows Event Log bookmark management
//! Persists position in each channel to avoid re-reading on restart
//! 
//! Bookmark strategy:
//! - Store ONLY XML string from WEVTAPI EvtQuery cursor
//! - Fallback: track last_record_id + last_event_ts
//! - On restart: load bookmark XML, pass to EvtQuery to resume
//! - Guarantee: WEVTAPI monotonic record IDs prevent re-reads
```

### After (windows/bookmark_manager.rs)
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

---

## Summary of Changes

| Component | Change | Impact |
|-----------|--------|--------|
| **Type System** | `EvidencePtr` → `Option<EvidencePtr>` | Compile-time enforcement |
| **Reader** | Remove placeholder, return `None` | No fake values |
| **Collector** | Remove from EvidencePtr field storage | No traps (record_id gone) |
| **Capture Writer** | Rewritten, use writer state + guard | Single source of truth |
| **Guard Assertion** | Added `is_some()` check | Runtime protection |
| **Documentation** | Bookmark labeled "CURSOR STATS ONLY" | Honest about capability |
| **Linux Sensors** | Changed to `None` | Consistent architecture |
| **Mock Fixtures** | Wrap with `Some()` | Match real behavior |

---

**All changes verified to compile cleanly and maintain correctness.**
