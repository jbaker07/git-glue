# OS Folder Organization Analysis - Violations & Reorganization Plan

## Current State Summary

### ✅ Correct Organization

**Windows (Proper Structure):**
```
windows/
├── sensors/
│   ├── capture_windows_rotating.rs  ✅ Capture binary in OS folder
│   ├── evtx_collector.rs            ✅ Emits core::Event
│   ├── sysmon_adapter.rs            ✅ Emits core::Event
│   ├── mod.rs
│   └── (13 sensor files)
└── signal_engine.rs                 ✅ Windows-specific detection

core/
├── event.rs                         ✅ Canonical Event + EvidencePtr contract
├── signal_result.rs                 ✅ Shared signal contract
└── (shared modules)

locald/                              ✅ Shared detection orchestrator
server/                              ✅ Shared UI + API
```

### ❌ Violations Found

**macOS (MISORGANIZED):**
```
macos/
├── sensors/
│   ├── capture_macos_rotating.rs    ✅ CORRECT: Capture binary here
│   └── (32 sensor files - some duplicates)
└── platform/
    ├── macos_fact_extractors.rs     ❌ SHOULD: Sensors emit core::Event
    ├── macos_fact_extractors_es.rs  ❌ SHOULD: Use core::Event + EvidencePtr
    ├── slot_engine.rs               ❌ DUPLICATE: Detection engine in OS folder
    └── (15 other files)

core/
├── macos_ingest.rs                  ❌ VIOLATION: OS-specific ingest in core
├── macos_stubs.rs                   ❌ VIOLATION: OS-specific stubs in core
├── macos_native_playbooks.rs        ❌ VIOLATION: OS-specific playbooks in core
```

**Linux (INCOMPLETE):**
```
linux/
├── ebpf/
│   └── (eBPF programs)
└── sensors/
    ├── capture_linux_rotating.rs    ❌ MISSING: No capture binary!
    └── (32 sensor files - same as macOS/Linux copies)

core/
└── (no linux_ingest.rs, linux_stubs.rs, etc. yet)
```

**Shared (Correct):**
```
locald/
├── edr_locald.rs                    ✅ Shared orchestrator
├── signal_persistence.rs            ✅ Shared persistence
└── (shared modules)

server/
├── ui_server.rs                     ✅ Shared API
└── api/
    ├── playbook_engine.rs           ✅ Shared
    └── (trust/decay engines)
```

---

## Violations List (by severity)

### CRITICAL VIOLATIONS

#### 1. macOS Detection Engine in Wrong Location
- **File:** `macos/platform/slot_engine.rs`
- **Issue:** Detection engine (like windows/signal_engine.rs) should NOT be in core/, but IS in OS-specific platform/ folder
- **Action:** Move to `macos/slot_engine.rs` (parallel to windows/signal_engine.rs)

#### 2. macOS-Specific Code in core/
- **Files:**
  - `core/macos_ingest.rs` - OS-specific ingest logic
  - `core/macos_stubs.rs` - OS-specific stubs
  - `core/macos_native_playbooks.rs` - OS-specific playbooks
- **Issue:** These should be in `macos/` folder, not core/
- **Action:** Move to `macos/macos_ingest.rs`, `macos/macos_stubs.rs`, `macos/macos_native_playbooks.rs`

#### 3. Linux Missing Capture Binary
- **File:** Missing `linux/sensors/capture_linux_rotating.rs`
- **Issue:** Windows and macOS have capture binaries; Linux doesn't
- **Expected:** `linux/sensors/capture_linux_rotating.rs` (reads /proc, ebpf, syscalls)
- **Action:** Create capture binary for Linux

#### 4. Duplicate Sensor Files (Massive Issue)
- **Problem:** `macos/sensors/` and `linux/sensors/` have identical filenames:
  - Both have: `alert_engine.rs`, `process_monitor.rs`, `network_watch.rs`, etc.
- **Issue:** These should be OS-specific implementations, not copies
- **Examples of duplicates:**
  - `alert_engine.rs` - appears in BOTH macos/sensors/ and linux/sensors/
  - `process_monitor.rs` - appears in BOTH
  - `mem_scan.rs` - appears in BOTH
  - `privilege_monitor.rs` - appears in BOTH
  - etc. (32 matching filenames!)
- **Action:** Create OS-specific implementations for each

### HIGH VIOLATIONS

#### 5. Fact Extractors Not Emitting canonical core::Event
- **Files:**
  - `macos/platform/macos_fact_extractors.rs` - extracts from BSM but output not verified
  - `macos/platform/macos_fact_extractors_es.rs` - extracts from ES but output not verified
- **Issue:** Should emit `core::Event + EvidencePtr`, not custom types
- **Action:** Verify they emit canonical events; move to `macos/sensors/` if they're sensor outputs

#### 6. macos/platform/ Should Be in macos/ Root
- **Folder:** `macos/platform/`
- **Issue:** "platform" is a vague name; files should be organized by function
- **Current bad structure:**
  ```
  macos/platform/
  ├── bsm_parser.rs
  ├── facts.rs
  ├── incidents.rs
  ├── playbooks.rs
  └── slot_engine.rs
  ```
- **Better structure:**
  ```
  macos/
  ├── sensors/
  │   ├── capture_macos_rotating.rs
  │   ├── bsm_parser.rs
  │   ├── facts.rs
  │   └── (sensor outputs)
  ├── slot_engine.rs
  └── (macos-specific files)
  ```
- **Action:** Flatten `macos/platform/` into `macos/` hierarchy

### MEDIUM VIOLATIONS

#### 7. Linux ebpf/ Should Use Shared core Types
- **Folder:** `linux/ebpf/`
- **Issue:** eBPF programs likely emit custom types, not core::Event
- **Action:** Create Linux sensors that convert eBPF output to core::Event + EvidencePtr

#### 8. Evidence Deref Duplication
- **Files:**
  - `macos/sensors/evidence_deref_example.rs`
  - `windows/sensors/evidence_deref_example.rs`
- **Issue:** Should be shared in `locald/` or `core/`, not duplicated
- **Action:** Move to `locald/evidence_deref.rs` (already exists!)

---

## Folder Structure Plan

### BEFORE (Current - Disorganized)
```
core/
├── macos_ingest.rs           ❌ OS-specific code
├── macos_stubs.rs            ❌ OS-specific code
├── macos_native_playbooks.rs ❌ OS-specific code
├── event.rs                  ✅ Shared
└── signal_result.rs          ✅ Shared

macos/
├── sensors/
│   ├── capture_macos_rotating.rs ✅
│   ├── alert_engine.rs           ❌ Duplicate with linux/
│   └── (32 files)
└── platform/
    ├── slot_engine.rs            ❌ Should be macos/
    ├── macos_fact_extractors.rs  ❌ Event emission unclear
    └── (15 files)

linux/
├── sensors/
│   ├── capture_linux_rotating.rs ❌ MISSING
│   ├── alert_engine.rs           ❌ Duplicate with macos/
│   └── (32 files)
└── ebpf/
    └── (eBPF programs)

locald/
├── edr_locald.rs             ✅
└── (shared modules)

server/
├── ui_server.rs              ✅
└── api/                       ✅
```

### AFTER (Target - Clean Organization)

```
core/
├── event.rs                  ✅ Canonical Event + EvidencePtr
├── signal_result.rs          ✅ Shared signal contract
├── (no OS-specific files!)
└── lib.rs

windows/
├── sensors/
│   ├── capture_windows_rotating.rs     ✅ Capture binary
│   ├── evtx_collector.rs               ✅ Emits core::Event
│   ├── sysmon_adapter.rs               ✅ Emits core::Event
│   ├── defender_adapter.rs             ✅ Emits core::Event
│   └── mod.rs
└── signal_engine.rs                    ✅ Windows-specific detection

macos/
├── sensors/
│   ├── capture_macos_rotating.rs       ✅ Capture binary
│   ├── bsm_parser.rs                   ✅ Parse → core::Event
│   ├── es_parser.rs                    ✅ Parse → core::Event
│   ├── process_monitor_macos.rs        ✅ macOS-specific impl
│   ├── network_watch_macos.rs          ✅ macOS-specific impl
│   └── mod.rs
├── slot_engine.rs                      ✅ macOS-specific detection
├── macos_ingest.rs                     ✅ (moved from core/)
├── macos_stubs.rs                      ✅ (moved from core/)
└── macos_native_playbooks.rs           ✅ (moved from core/)

linux/
├── sensors/
│   ├── capture_linux_rotating.rs       ✅ Capture binary (NEW!)
│   ├── ebpf_collector.rs               ✅ eBPF → core::Event
│   ├── procfs_parser.rs                ✅ /proc → core::Event
│   ├── process_monitor_linux.rs        ✅ Linux-specific impl
│   ├── network_watch_linux.rs          ✅ Linux-specific impl
│   └── mod.rs
├── ebpf/
│   └── (eBPF programs)
└── (linux-specific files)

locald/
├── edr_locald.rs             ✅ Shared orchestrator (no OS-specific code)
├── signal_persistence.rs     ✅ Shared
├── signal_orchestrator.rs    ✅ Shared
└── playbook_loader.rs        ✅ Shared

server/
├── ui_server.rs              ✅ Shared
└── api/                       ✅ Shared
```

---

## File Movement Commands (Phase 1: macOS)

### Move macOS OS-specific code out of core/

```bash
# Move to macos/
mv core/macos_ingest.rs macos/
mv core/macos_stubs.rs macos/
mv core/macos_native_playbooks.rs macos/
```

### Flatten macos/platform/ → macos/

```bash
# Move key files up
mv macos/platform/slot_engine.rs macos/
mv macos/platform/bsm_parser.rs macos/sensors/
mv macos/platform/macos_fact_extractors.rs macos/sensors/
mv macos/platform/macos_fact_extractors_es.rs macos/sensors/

# Keep supporting files in macos/ root or organize by function
# (Other platform files to be reviewed)
```

### Remove duplicate evidence_deref in sensors

```bash
# Already exists in locald/evidence_deref_example.rs
rm macos/sensors/evidence_deref_example.rs
rm windows/sensors/evidence_deref_example.rs
```

---

## File Movement Commands (Phase 2: Linux)

### Create Linux capture binary (stub)

```bash
# Need to create: linux/sensors/capture_linux_rotating.rs
```

### Segregate Linux-specific sensor implementations

```bash
# Rename to make OS affinity explicit:
mv linux/sensors/process_monitor.rs linux/sensors/process_monitor_linux.rs
mv linux/sensors/network_watch.rs linux/sensors/network_watch_linux.rs
# ... (32 sensors need OS-specific implementations)
```

---

## Verification Checklist

### Phase 1: Violations
- [ ] List all files violating organization (DONE - see above)
- [ ] Identify duplicates between OS folders
- [ ] Identify OS-specific code in core/

### Phase 2: Reorganization (NO COMPILATION)
- [ ] Move OS-specific files from core/ to OS folders
- [ ] Move detection engines to OS roots (not core/)
- [ ] Flatten macos/platform/ structure
- [ ] Create capture_linux_rotating.rs stub

### Phase 3: Verification (After Reorganization)
- [ ] Verify sensor mod.rs exports match new locations
- [ ] Verify locald/edr_locald.rs imports still work
- [ ] Check that core/ has ONLY shared types (Event, EvidencePtr, etc.)
- [ ] Verify no OS-specific code in core/lib.rs exports

---

## Summary of Violations

| File | Location | Issue | Action |
|------|----------|-------|--------|
| `slot_engine.rs` | macos/platform/ | Detection engine in platform folder | Move to macos/ |
| `macos_ingest.rs` | core/ | OS-specific code in core | Move to macos/ |
| `macos_stubs.rs` | core/ | OS-specific code in core | Move to macos/ |
| `macos_native_playbooks.rs` | core/ | OS-specific code in core | Move to macos/ |
| `capture_linux_rotating.rs` | MISSING | No Linux capture binary | Create in linux/sensors/ |
| `*_deref_example.rs` | macos/sensors/ + windows/sensors/ | Duplicate shared file | Remove, keep in locald/ |
| `alert_engine.rs` | macos/sensors/ + linux/sensors/ | Duplicate names, should be OS-specific | Rename/implement separately |
| `platform/` | macos/platform/ | Bad folder name | Flatten into macos/ |
| 32 sensor files | macos/sensors/ + linux/sensors/ | Same names in both = copies | Make OS-specific implementations |

**TOTAL VIOLATIONS: 40+ files requiring reorganization**
