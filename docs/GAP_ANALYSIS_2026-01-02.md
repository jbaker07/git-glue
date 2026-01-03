# GAP ANALYSIS: Incident Compiler Pipeline
## Senior Engineer Review – Evidence-Grounded Assessment

**Date:** 2026-01-02  
**Scope:** Full pipeline audit from sensor → incident, grounded in actual repo code

---

## 1. Current End-to-End Architecture (As Implemented)

### Dataflow
```
┌────────────────┐    ┌────────────────┐    ┌────────────────┐    ┌────────────────┐
│   OS Sensors   │───▶│  Capture Bin   │───▶│    locald      │───▶│   ui_server    │
│  (per-platform)│    │ (rotating seg) │    │ (orchestrator) │    │ (axum REST)    │
└────────────────┘    └────────────────┘    └────────────────┘    └────────────────┘
      │                     │                      │                      │
      ▼                     ▼                      ▼                      ▼
 Event emission       segments/*.jsonl       SignalResult          /api/incidents
 via collect()        + index.json           + hypothesis          /api/explain
```

### Binaries (Active)

| Binary | Location | Purpose |
|--------|----------|---------|
| `capture_linux_rotating` | [src/bin/capture_linux_rotating.rs](src/bin/capture_linux_rotating.rs) | eBPF ringbuf drain → segments/*.jsonl |
| `capture_macos_rotating` | [macos/capture_macos_rotating.rs](macos/capture_macos_rotating.rs) | ES/BSM → segments/*.jsonl |
| `capture_windows_rotating` | [src/bin/capture_windows_rotating.rs](src/bin/capture_windows_rotating.rs) | WEVTAPI → segments/*.jsonl |
| `ui_server` | [crates/server/src/main.rs](crates/server/src/main.rs) | REST API + incident UI |
| `extract_evidence` | [src/bin/extract_evidence.rs](src/bin/extract_evidence.rs) | Deref evidence pointers |

### On-Disk Artifacts

| Path | Content |
|------|---------|
| `/var/lib/edr/segments/*.jsonl` | Raw telemetry segments (JSONL, rotate at size/time) |
| `/var/lib/edr/segments/index.json` | Segment manifest with SHA256 checksums |
| `/var/lib/edr/baselines/baselines_<host>.json` | Per-host baseline state |
| `/var/lib/edr/incidents/<ns>/*.jsonl` | Incident persistence per namespace |
| `/var/lib/edr/bundles/` | Support bundle export directory |

---

## 2. Canonical Event/Facts Model (As Implemented)

### Event Struct – `edr_core::Event`
**Source:** [crates/core/src/event.rs](crates/core/src/event.rs)

```rust
pub struct Event {
    pub ts_ms: i64,               // Epoch milliseconds
    pub host: String,             // Machine identifier
    pub tags: Vec<String>,        // Routing tags (process, network, etc.)
    pub proc_key: Option<String>, // Process scope key
    pub file_key: Option<String>, // File scope key (or socket)
    pub identity_key: Option<String>, // User identity scope key
    pub evidence_ptr: Option<EvidencePtr>, // Back-pointer to segment
    pub fields: BTreeMap<String, Value>,   // Canonical fields
}
```

### Canonical Event Types (9 Validated)
**Source:** [crates/core/src/event.rs](crates/core/src/event.rs#L45-L98) – `validate_canonical_primitive()`

| Type | Required Fields |
|------|-----------------|
| `credential_access` | proc_key, method |
| `discovery` | proc_key |
| `exfiltration` | file_key OR dest_ip |
| `network_connection` | dest_ip, dest_port |
| `persistence_change` | path |
| `defense_evasion` | proc_key, method |
| `process_injection` | proc_key, target_proc_key |
| `auth_event` | identity_key, auth_type |
| `script_exec` | proc_key, script_type |

### Canonical Field Keys (~100+)
**Source:** [crates/core/src/event_keys.rs](crates/core/src/event_keys.rs)

```
PROC_: pid, ppid, uid, euid, exe, argv, cmdline, capabilities, signer
FILE_: path, inode, sha256, md5, size, entropy, permissions
NET_:  remote_ip, remote_port, local_ip, local_port, proto, socket_inode
AUTH_: user, logon_type, source_ip, target_user, privileges
MEM_:  address, size, protection, is_wx_violation
REG_:  key_path, value_name, value_data (Windows)
```

### CanonicalEvent (Hypothesis System)
**Source:** [locald/hypothesis/canonical_event.rs](locald/hypothesis/canonical_event.rs)

Rich event model with typed contexts:

```rust
pub struct CanonicalEvent {
    pub evidence_ptr: EvidencePtr,
    pub event_type: CanonicalEventType,  // ~30 typed variants
    pub host_id: String,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub process: ProcessContext,     // 20+ fields
    pub file: Option<FileContext>,   // 10+ fields
    pub network: Option<NetworkContext>,
    pub memory: Option<MemoryContext>,
    pub registry: Option<RegistryContext>,
    pub auth: Option<AuthContext>,
    pub os_context: OsSpecificContext, // Windows ETW / macOS ES / Linux eBPF
}
```

### Fact Types (18 Implemented)
**Source:** [locald/hypothesis/canonical_fact.rs](locald/hypothesis/canonical_fact.rs)

| Domain | Fact Types |
|--------|------------|
| **Process** | ProcSpawn, Exec, ShellCommand, ScriptExec |
| **Network** | OutboundConnect, InboundConnect, DnsResolve |
| **File** | WritePath, ReadPath, CreatePath, DeletePath, RenamePath |
| **Memory** | MemWX, MemAlloc, Injection |
| **Auth** | AuthEvent, PrivilegeBoundary |
| **Persist** | PersistArtifact, RegistryMod |
| **Tamper** | LogTamper, SecurityToolDisable |
| **Module** | ModuleLoad |

---

## 3. Join Key & Fidelity Audit (THE CORE)

### Scope Key Types
**Source:** [locald/hypothesis/scope_keys.rs](locald/hypothesis/scope_keys.rs)

```rust
pub enum ScopeKey {
    Process { key: String },  // hash(host + boot_id + start_time + pid + exe_hash)
    User { key: String },     // host + user identity
    Executable { key: String }, // host + exe hash
    Socket { key: String },   // network endpoint identity
    File { key: String },     // stable file identity
    Campaign { key: String }, // cross-host exe_hash grouping
    Session { key: String },  // user session identity
}
```

### Join Key Fidelity Matrix

| Fact Type | Windows | macOS | Linux | Primary Join Key | Secondary Join Keys | Fidelity |
|-----------|---------|-------|-------|------------------|---------------------|----------|
| **ProcSpawn** | ✅ YES | ✅ YES | ✅ YES | parent_proc_key, child_proc_key | host_id, ts | **HARD** |
| **Exec** | ✅ YES | ✅ YES | ✅ YES | proc_key + exe_hash | signer, cmdline | **HARD** |
| **OutboundConnect** | ✅ YES | ✅ YES | ✅ YES | proc_key + dst_ip:dst_port | sock_id | **HARD** |
| **InboundConnect** | ⚠️ PARTIAL | ✅ YES | ✅ YES | proc_key + src_ip:src_port | sock_id | **SOFT** (Windows limited) |
| **DnsResolve** | ⚠️ PARTIAL | ⚠️ PARTIAL | ⚠️ PARTIAL | proc_key + query | responses | **SOFT** (attribution gaps) |
| **WritePath** | ✅ YES | ✅ YES | ✅ YES | proc_key + path | inode, entropy | **HARD** |
| **ReadPath** | ✅ YES | ✅ YES | ✅ YES | proc_key + path | inode | **HARD** |
| **MemWX** | ✅ YES | ⚠️ PARTIAL | ✅ YES | proc_key + addr | prot_before, prot_after | **HARD** (macOS ES limited) |
| **MemAlloc** | ✅ YES | ⚠️ PARTIAL | ✅ YES | proc_key + addr | size, protection | **HARD** |
| **Injection** | ✅ YES | ⚠️ PARTIAL | ✅ YES | source_proc_key → target_proc_key | injection_type | **SOFT** (macOS lacks dyld detail) |
| **AuthEvent** | ✅ YES | ✅ YES | ✅ YES | identity_key + auth_type | source_ip | **HARD** |
| **PrivilegeBoundary** | ✅ YES | ✅ YES | ✅ YES | proc_key | uid_before, uid_after | **HARD** |
| **PersistArtifact** | ✅ YES | ✅ YES | ✅ YES | path_or_key | artifact_type | **HARD** |
| **RegistryMod** | ✅ YES (native) | ❌ N/A | ❌ N/A | key + value_name | operation | **HARD** (Windows only) |
| **LogTamper** | ✅ YES | ✅ YES | ✅ YES | log_type | action | **HARD** |
| **ModuleLoad** | ✅ YES | ✅ YES | ✅ YES | proc_key + path | hash, signer | **HARD** |

### Gap Analysis: Missing/Weak Join Keys

#### 🔴 CRITICAL GAP: DNS Resolution Attribution
- **Problem:** DNS queries often lack reliable process attribution
- **Windows:** ETW `Microsoft-Windows-DNS-Client` has process ID but unreliable
- **macOS/Linux:** System resolver abstracts away calling process
- **Impact:** Cannot reliably chain `Exec → DnsResolve → OutboundConnect`
- **Mitigation:** Use timing correlation + destination IP matching (soft join)

#### 🟡 HIGH GAP: Cross-Process Injection on macOS
- **Problem:** macOS Endpoint Security has limited dyld interpose visibility
- **Current:** Only detect task_for_pid, mach_vm_* but miss dyld injection
- **Impact:** Cannot distinguish malicious dyld from legit frameworks
- **Mitigation:** Supplement with file hash baseline for known-good interposers

#### 🟡 HIGH GAP: Thread-Level Granularity
- **Problem:** Thread ID (tid) captured but not used in scope keys
- **Current:** Process-level scope keys only
- **Impact:** Multi-threaded injection techniques may alias
- **Mitigation:** Add thread scope key variant for injection facts

---

## 4. Capability Contract Proposal

### capabilities.json Schema
**Reference:** [crates/integration/examples/capabilities_manifest.yaml](crates/integration/examples/capabilities_manifest.yaml)

```json
{
  "version": "1.0",
  "host_id": "<machine_id>",
  "collected_at": "<ISO8601>",
  "collectors": {
    "windows_etw": {
      "enabled": true,
      "providers": ["Microsoft-Windows-Sysmon", "Microsoft-Windows-Security-Auditing"],
      "health": "healthy",
      "drop_rate": 0.001
    },
    "macos_es": {
      "enabled": true,
      "events": ["exec", "fork", "open", "mmap", "mprotect"],
      "health": "healthy"
    },
    "linux_ebpf": {
      "enabled": true,
      "programs": ["execve", "connect4", "mmap"],
      "health": "healthy"
    }
  },
  "fact_coverage": {
    "ProcSpawn": { "supported": true, "fidelity": "hard" },
    "Exec": { "supported": true, "fidelity": "hard" },
    "OutboundConnect": { "supported": true, "fidelity": "hard" },
    "DnsResolve": { "supported": true, "fidelity": "soft", "gap": "attribution" },
    "MemWX": { "supported": true, "fidelity": "hard" },
    "Injection": { "supported": true, "fidelity": "soft", "gap": "dyld_detail" }
  },
  "critical_gaps": []
}
```

### Runtime Integration
**Source:** [locald/hypothesis/visibility.rs](locald/hypothesis/visibility.rs)

Existing `CollectorStatus` and `WindowVisibility` already track:
- `health: CollectorHealth` (Healthy/Degraded/Unhealthy/Offline)
- `drop_rate: f64`
- `high_watermark / low_watermark: DateTime<Utc>`
- `error_messages: Vec<String>`

**Proposal:** Add `capabilities.json` emission on startup + periodic refresh:
1. `capture_*_rotating` emits capabilities.json to telemetry_root on start
2. `locald` reads capabilities on pipeline init
3. Hypothesis maturity calculation consumes capability fidelity (already wired via `claim_confidence()`)

---

## 5. Differentiation Inventory

### What's Already Differentiated

| Feature | Status | Evidence |
|---------|--------|----------|
| **Deterministic Incident IDs** | ✅ Implemented | [incident.rs](locald/hypothesis/incident.rs#L180) – SHA256(host + scope + family + time_bucket) |
| **Evidence Pointer Integrity** | ✅ Implemented | [canonical_event.rs](locald/hypothesis/canonical_event.rs#L20) – EvidencePtr with SHA256 |
| **Visibility Gap Accounting** | ✅ Implemented | [visibility.rs](locald/hypothesis/visibility.rs) – Full collector health tracking |
| **Tier-0 Invariants (Unsuppressible)** | ✅ Implemented | [tier0.rs](locald/hypothesis/tier0.rs) – 10 invariant types |
| **Multi-Domain Corroboration** | ✅ Implemented | [promotion.rs](locald/hypothesis/promotion.rs#L65) – FactDomain enum + scoring |
| **PID Reuse Safe Scope Keys** | ✅ Implemented | [scope_keys.rs](locald/hypothesis/scope_keys.rs#L85) – boot_id + start_time in hash |
| **Namespace Isolation** | ✅ Implemented | [ui_server.rs](server/ui_server.rs#L742) – validate_namespace_access() |
| **Cross-Platform Canonical Model** | ✅ Implemented | [canonical_fact.rs](locald/hypothesis/canonical_fact.rs) – 18 fact types |

### TOP 2 Missing Join Keys (Hero Chain Impact)

#### 1. **DNS → Network Correlation Key**
- **Gap:** No `dns_query_id` to correlate DnsResolve → OutboundConnect
- **Hero Chain Broken:** `Exec(curl) → DnsResolve(evil.com) → OutboundConnect(93.184.216.34)`
- **Fix:** Add `dns_correlation_id` field + timing window matcher in hypothesis

#### 2. **Thread-Level Injection Scope**
- **Gap:** `target_proc_key` exists but no `target_thread_key`
- **Hero Chain Broken:** `Exec(malware) → CreateRemoteThread(tid=123) → MemWX(in-thread)`
- **Fix:** Add `ThreadScope { proc_key, tid, start_time }` for Injection facts

### Hero Chain: Full-Fidelity Attack Narrative (Goal State)

```
ProcessCreate(pid=1234, exe="malware.exe", parent="explorer.exe")
    ↓ proc_key: "a1b2c3..."
MemWX(addr=0x7fff0000, prot=RWX)
    ↓ same proc_key
OutboundConnect(dst=93.184.216.34:443)
    ↓ same proc_key
DnsResolve(query="evil.com", response=["93.184.216.34"])
    ↓ **MISSING HARD JOIN** → timing correlation only
Injection(target_proc="lsass.exe", type=RemoteThread)
    ↓ source_proc_key → target_proc_key
CredentialAccess(method="mimikatz", target="LSASS")
    ↓ target_proc_key
```

**Current Coverage:** 5/6 links are HARD joins, 1/6 (DNS) is SOFT

---

## 6. Wazuh + Open Stack Integration Recommendation

### Analysis: Export vs. Ingest

| Direction | Pros | Cons |
|-----------|------|------|
| **Export TO Wazuh** | Leverage existing SIEM infrastructure, compliance mapping | Lose incident compiler context, double-storage |
| **Ingest FROM Wazuh** | Enrich with network/cloud visibility | Wazuh events lack our scope keys |

### Recommendation: **Bi-directional with Clear Boundaries**

1. **Export Incidents (not raw events)** to Wazuh
   - Format: Wazuh `alert` JSON schema
   - Include: incident_id, severity, timeline summary, MITRE mapping
   - Exclude: Raw segments (storage explosion)

2. **Ingest Network Flow/Cloud Events** from Wazuh
   - Format: Our `CanonicalEvent` with `VendorAlert` fact type
   - Use case: Enrich DNS correlation, cloud API anomalies
   - Join key: IP address + timestamp window (soft join)

### Integration Points

```yaml
# Proposed: wazuh_integration.yaml
export:
  enabled: true
  target: "wazuh-manager:1515"
  format: "wazuh_alert"
  filter:
    min_severity: "medium"
    include_tier0: true
  
ingest:
  enabled: true
  sources:
    - type: "wazuh_syslog"
      port: 1514
      fact_type: "VendorAlert"
      join_strategy: "ip_time_window"
      window_seconds: 300
```

---

## 7. Deliverable Plan (Next 7 Days)

### Day 1-2: Capability Contract Foundation

| Task | Priority | Files | Output |
|------|----------|-------|--------|
| Add `capabilities.json` emission to capture binaries | P0 | `capture_*_rotating.rs` | capabilities.json on startup |
| Wire capability reading into `HypothesisController` | P0 | `hypothesis_controller.rs` | `CollectorStatus` populated from capabilities |
| Add `fact_coverage` to `WindowVisibility` | P1 | `visibility.rs` | Per-fact fidelity tracking |

### Day 3-4: DNS Correlation Enhancement

| Task | Priority | Files | Output |
|------|----------|-------|--------|
| Add `dns_correlation_id` field to Event | P0 | `event_keys.rs`, `event.rs` | New canonical field |
| Implement DNS→Network timing matcher | P1 | `hypothesis_controller.rs` | Soft join with confidence score |
| Add DnsResolve → OutboundConnect hypothesis rule | P1 | `playbooks/common/dns_egress.yaml` | New playbook |

### Day 5-6: Thread-Level Scope Key

| Task | Priority | Files | Output |
|------|----------|-------|--------|
| Add `Thread` variant to `ScopeKey` enum | P1 | `scope_keys.rs` | `Thread { proc_key, tid, start_time }` |
| Update Injection fact to use thread scope | P1 | `canonical_fact.rs` | `target_thread_key` field |
| Wire thread context in Windows signal engine | P2 | `windows/signal_engine.rs` | TID extraction from ETW |

### Day 7: Wazuh Export Prototype

| Task | Priority | Files | Output |
|------|----------|-------|--------|
| Add `WazuhExporter` sink | P2 | `crates/locald/src/pipeline.rs` | New sink implementation |
| Implement incident → Wazuh alert mapping | P2 | `crates/locald/src/wazuh_export.rs` | JSON translation |
| Add export config to `locald.yaml` | P2 | `config/locald.yaml` | Wazuh section |

---

## Summary: Critical Path

```
1. capabilities.json (visibility contract)     ← Day 1-2
2. DNS correlation (biggest join gap)          ← Day 3-4  
3. Thread scope key (injection fidelity)       ← Day 5-6
4. Wazuh export (enterprise integration)       ← Day 7
```

**Total estimated effort:** 5-7 engineer-days

**Risk:** DNS correlation is inherently soft; accept ~80% confidence for this join.

---

## Appendix: File References

| Section | Key Files |
|---------|-----------|
| Event Model | [crates/core/src/event.rs](crates/core/src/event.rs), [crates/core/src/event_keys.rs](crates/core/src/event_keys.rs) |
| Canonical Event | [locald/hypothesis/canonical_event.rs](locald/hypothesis/canonical_event.rs) |
| Canonical Fact | [locald/hypothesis/canonical_fact.rs](locald/hypothesis/canonical_fact.rs) |
| Scope Keys | [locald/hypothesis/scope_keys.rs](locald/hypothesis/scope_keys.rs) |
| Visibility | [locald/hypothesis/visibility.rs](locald/hypothesis/visibility.rs) |
| Promotion Math | [locald/hypothesis/promotion.rs](locald/hypothesis/promotion.rs) |
| Tier-0 Invariants | [locald/hypothesis/tier0.rs](locald/hypothesis/tier0.rs) |
| Incident Model | [locald/hypothesis/incident.rs](locald/hypothesis/incident.rs) |
| Signal Engines | [crates/locald/src/os/](crates/locald/src/os/) |
| Capture Binaries | [src/bin/capture_*.rs](src/bin/), [macos/capture_macos_rotating.rs](macos/capture_macos_rotating.rs) |
| Playbooks | [playbooks/](playbooks/) |
