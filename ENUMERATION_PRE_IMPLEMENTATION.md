# Ground Truth Enumeration (Pre-Implementation)

## Current State Snapshot (2025-12-24)

### 1. Capture Heartbeat Schemas

#### macOS (`macos/capture_macos_rotating.rs:609-635`)
**File**: macos/capture_macos_rotating.rs, Line 609-635
**Keys Written**: 
- `ts_ms`, `pid`, `seq`, `schema_version`, `transport`
- `events_read_total`, `parse_failed_total`, `parse_failed_malformed`, `parse_failed_unsupported`, `parse_failed_validation`, `evidence_ptr_overwrite_total`
- **Canonical counters (10)**: `cred_access_count`, `discovery_exec_count`, `archive_tool_exec_count`, `staging_write_count`, `net_connect_count`, `persistence_change_count`, `defense_evasion_count`, `process_injection_count`, `auth_event_count`, `script_exec_count`
- `capture_build_id`

**NOTE**: Uses legacy heartbeat names (cred_access_count, discovery_exec_count, net_connect_count) — will be remapped by ui_server.rs health endpoint.

#### Linux (`linux/capture_linux_rotating.rs:361-389`)
**File**: linux/capture_linux_rotating.rs, Line 361-389
**Keys Written**:
- `ts_ms`, `pid`, `segment_id`, `schema_version`, `transport`
- `ebpf_read`, `sensors_read`, `decode_failed`
- Privilege counters: `priv_uid_change_count`, `priv_gid_change_count`, `priv_cap_change_count`, `priv_boundary_cross_count`
- Other sensors: `net_connect_count`, `remote_tool_exec_count`, `log_tamper_count`, `audit_tamper_count`, `c2_suspected_count`
- **Canonical counters (10)**: `cred_access_count`, `discovery_exec_count`, `archive_tool_exec_count`, `staging_write_count`, `net_connect_prim_count`, `persistence_change_count`, `defense_evasion_count`, `process_injection_count`, `auth_event_count`, `script_exec_count`

**NOTE**: Uses legacy names but includes all 10 counters.

#### Windows (`windows/capture_windows_rotating.rs:372-387`)
**File**: windows/capture_windows_rotating.rs, Line 372-387
**CRITICAL GAP**: Windows heartbeat writes per-source metrics but does NOT include canonical counters!
**Keys Written**: 
- `ts`, `transport` (`"evtx"`)
- `source`, `events_read_total`, `events_read_delta`
- `parse_failed_total`, `parse_failed_delta`
- `last_event_ts`, `status`
- **MISSING**: All canonical event type counters

### 2. HealthResponse Schema

**File**: server/ui_server.rs, Line 39-63
**Canonical Fields (9)**: 
- `credential_access_count`
- `discovery_count`
- `exfiltration_count`
- `network_connection_count`
- `persistence_change_count`
- `defense_evasion_count`
- `process_injection_count`
- `auth_event_count`
- `script_exec_count`

**Optional Breakdown (2)**:
- `archive_tool_exec_count`
- `staging_write_count`

**Extraction Logic** (ui_server.rs:247-315):
- Reads `capture_heartbeat.json` (macOS/Linux) or `heartbeat_*.json` (Windows)
- Remaps legacy names: `cred_access_count` → `credential_access_count`, etc.
- Derives: `exfiltration_count = archive_tool_exec_count + staging_write_count`
- Defaults to 0 if missing (backwards compat)

### 3. UI KPI Mapping

**File**: ui/app.js, Line 696-718
**Displayed (9 KPIs)**:
```javascript
credential_access_count → "Credential Access"
discovery_count → "Discovery"
exfiltration_count → "Exfiltration"
network_connection_count → "Network"
persistence_change_count → "Persistence"
defense_evasion_count → "Evasion"
process_injection_count → "Injection"
auth_event_count → "Auth"
script_exec_count → "Scripts"
```

### 4. Primitive Derivation Paths

#### macOS
**Location**: macos/sensors/bsm/bsm_reader.rs:167
**Function**: `pub fn derive_primitive_events(base_event: &Event) -> Vec<Event>`
**Calls per-type derivers**: credential_access.rs, discovery.rs, exfiltration.rs, etc. (all exist)
**Wiring**: Called in capture_macos_rotating.rs during event processing

#### Linux
**Location**: linux/sensors/ebpf_primitives/mod.rs:20
**Function**: `pub fn derive_primitive_events(base_event: &Event) -> Vec<Event>`
**Calls per-type derivers**: credential_access.rs, discovery.rs, exfiltration.rs, etc. (all exist)
**Wiring**: Called in capture_linux_rotating.rs during event processing

#### Windows
**Location**: windows/sensors/primitives/mod.rs:20
**Function**: `pub fn derive_primitive_events(base_event: &Event) -> Vec<Event>`
**Calls per-type derivers**: credential_access.rs, discovery.rs, exfiltration.rs, etc. (all exist)
**Wiring**: Called in capture_windows_rotating.rs during event processing

### 5. Canonical Type Definitions

**Core Location**: core/src/event.rs
**Canonical Types (9)**:
1. credential_access
2. discovery
3. exfiltration
4. network_connection
5. persistence_change
6. defense_evasion
7. process_injection
8. auth_event
9. script_exec

**Validation**: `validate_canonical_primitive()` enforces required fields per type

### 6. Evidence Pointer Invariant

**Enforced in**: All capture writers (macos/linux/windows/capture_*_rotating.rs)
**Pattern**: 
- Sensors/derivers set `evidence_ptr: None`
- Capture writers call `assign_evidence_ptrs()` after `sort()` and `dedup()`
- Only then written to segment

---

## Deficiency Analysis

### EXISTING ✅
- ✅ All 9 canonical sensor types defined
- ✅ Primitive derivation functions exist for all OS
- ✅ Heartbeat schemas include counters (macOS/Linux)
- ✅ HealthResponse struct has 9 canonical + 2 breakdown fields
- ✅ UI displays 9 KPIs
- ✅ EvidencePtr invariant enforced in capture writers
- ✅ validate_canonical_primitive() exists

### MISSING / INCOMPLETE ❌

#### A) Contract Lock + Parity Test
- [ ] No test ensuring consistency across: macOS heartbeat → HealthResponse → UI
- [ ] No single source of truth for counter field names
- [ ] Windows heartbeat missing canonical counters entirely

#### B) E2E Golden Trace Harness
- [ ] No integration test feeding synthetic events through full derivation pipeline
- [ ] No test asserting all 9 types can be produced and validated
- [ ] No manual commands documented to trigger each type

#### C) High-Value Detection Upgrades
- [x] Credential access (basic: tool runs, ssh keys)
- [ ] Persistence (needs platform-specific paths, file, registry, service checks)
- [ ] Process injection (needs correlation with defense_evasion)
- [ ] **No correlation detectors** (multiple facts + TTL)
- [ ] **No behavioral detectors** beyond primitive patterns

#### D) Ground Truth Playbooks
- [ ] No YAML playbooks using canonical primitives
- [ ] No incident correlation rules (e.g., cred→discovery→persistence)
- [ ] No playbook loader wiring (if playbook system exists)

#### E) Demo Runner
- [ ] No command/script to trigger all 9 primitive types
- [ ] No verification of UI health endpoint updates
- [ ] No end-to-end runner per OS

---

## Next Steps (Implementation Order)

1. **Fix Windows Heartbeat** — Add canonical counters to Windows capture
2. **Contract Parity Test** — Rust test validating all 3 heartbeatschemas + HealthResponse
3. **E2E Golden Trace** — Integration test with synthetic events
4. **High-Value Detectors** — 3+ per OS (stronger than existing primitives)
5. **Ground Truth Playbooks** — YAML + loader wiring
6. **Demo Runner** — Cross-platform verification script

---
