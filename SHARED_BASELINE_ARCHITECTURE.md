# Shared Baseline System vs Per-OS Design

## The Design Decision

### ❌ What We Avoided: Per-OS Baselines

```
Windows Detector               macOS Detector               Linux Detector
    ↓                             ↓                           ↓
Windows Baseliner          macOS Baseliner             Linux Baseliner
(understands 4624,         (understands FSEvents,      (understands eBPF,
 Sysmon, WMI)              launchd, FSEvents)          syscalls)
    ↓                             ↓                           ↓
3 separate baseline         3 separate persistence      3 separate learned
engines + DBs              policies                     state models
```

**Problems:**
- Triples code maintenance burden
- Each gets different feature sets (Windows richer than macOS/Linux)
- Shared incident policy becomes complex (do all 3 freeze? which sync?)
- New feature → implement 3 times
- Testing bloats

### ✅ What We Built: Shared Baseline Engine

```
Windows Capture        macOS Capture            Linux Capture
   ↓                      ↓                        ↓
forensic_hooks::Event  forensic_hooks::Event   forensic_hooks::Event
   ↓                      ↓                        ↓
     ┌────────────────────┼────────────────────┐
     │                                        │
     ↓                                        ↓
Shared Baseline Engine (locald/baseline/)    Windows Signal Engine
  ├─ update_first_seen()  ← canonical Event  (understands context)
  ├─ update_rates()       ← canonical Event  ├─ Admin Share rules
  └─ update_feature_vector() ← canonical Event  ├─ Remote Logon rules
     ↓                                        ├─ Sysmon context
  One baseline_store                         └─ consults baseline
  (per host)
     ↓
  telemetry_root/baselines/
  baselines_<host>.json

  Future: macOS Signal Engine, Linux Signal Engine
  (each OS builds what makes sense for that OS)
```

**Benefits:**
- **One baseline per host** regardless of OS
- **OS-specific detectors are thin** (extract event context, consult baseline)
- **Baseline updates are identical** across platforms (no divergence)
- **Persistence, freezing, and policy** are centralized
- **New feature** → add one update method, one query method

## Key Design Principles

### 1. Canonical Events Decouple OS Specifics from Baselining

**Per-OS approach:**
```rust
// Windows baseliner must understand 4624 event log parsing
if event_id == 4624 {
    parse_event_log_xml();
    // 20 lines of Windows-specific code
}

// macOS baseliner must understand FSEvents
if path == "/var/log/system.log" {
    // 20 lines of macOS-specific code
}
```

**Shared approach:**
```rust
// BaselineUpdater receives already-parsed canonical Event
fn update_first_seen(baseline: &mut HostBaseline, event: &forensic_hooks::Event) {
    // Extract from event.fields (BTreeMap)
    if let Some(dest_ip) = event.fields.get("dest_ip") {
        baseline.first_seen.remote_ips_per_exe.entry(exe_key)
            .or_insert_with(HashSet::new)
            .insert(dest_ip.clone());
    }
    // Same 5 lines for all OSes
}
```

**Result:** Each OS **capture** implements parsing once. **Baseline** stays pure.

### 2. Keying by (host, context) Not by OS

All queries are:
- `is_new_remote_ip(baseline, exe_key, ip)` ← exe_key works for all OSes
- `is_burst(baseline, "admin_share_access:per_host", rate)` ← metric name is portable
- `is_feature_vector_anomalous(baseline, "network", features)` ← vector type is portable

**No OS-specific keys like:**
- `is_new_remote_ip_windows()`
- `is_burst_macos()`
- `is_feature_vector_anomalous_linux()`

### 3. Tier-0 Invariants Never Use Baseline

Behavior that should **always alert**:

| Signal | Invariant | Baseline Use |
|--------|-----------|--------------|
| LSASS access + credential dumping | Yes | None |
| CreateRemoteThread → sensitive syscall | Yes | None |
| Log wipe + audit disable sequence | Yes | None |
| Agent binary modified on disk | Yes | None |
| Injection → execute in unusual process | Yes | None |

These are **Tier-0**: alert first, correlate later.

| Signal | Invariant | Baseline Use |
|--------|-----------|--------------|
| Admin share burst | No (rate varies) | ✅ Consult EMA |
| New remote IP for exe | No (depends on exe) | ✅ Consult first-seen |
| LOLBin invocation | No (varies by org) | ✅ Consult first-seen |
| High-rate process spawning | No (rate varies) | ✅ Consult EMA |

These are **Tier-2/3**: weight with baseline, escalate on corroboration.

### 4. First-Seen Sets Have Zero Learning Period

```rust
// Event 1: powershell.exe connects to 192.168.1.1
// → immediately stored in baseline

// Event 2: powershell.exe connects to 192.168.1.1 again
// → known connection, no alert

// Event 3: powershell.exe connects to 10.0.0.99
// → NEW connection, potential alert (if Tier-2/3)
```

No need for "wait 48 hours for learning"—first-seen uses **opt-in flagging**, not **opt-out baselining**.

### 5. Rate & Feature Baselines Use Gradual EMA

```rust
// Minute 1: 5 admin shares accessed → ema update
// Minute 2: 6 admin shares accessed → ema update
// Minute 50: ema ≈ 5.5, ema_var ≈ 0.2

// Minute 51: spike to 100 admin shares
// threshold = 5.5 + 2.0 * sqrt(0.2) ≈ 6.4
// 100 > 6.4 → BURST ✅
```

EMA naturally handles **seasonal variations** and **gradual load increases** without explicit hour×weekday bucketing.

## Deployment Sequence

### Phase 1: Baseline Infrastructure ✅ Done
- [x] locald/baseline/types.rs
- [x] locald/baseline/baseline_store.rs
- [x] locald/baseline/baseline_update.rs
- [x] locald/baseline/baseline_query.rs
- [x] Module exports

### Phase 2: Integrate into locald Event Loop ⏳ Next
```rust
// In edr_locald main()
let mut baseline_store = BaselineStore::new(telemetry_root.join("baselines"));
baseline_store.load_all();

// In ingest_segment() for each canonical event
if let Some(mut baseline) = baseline_store.get_or_create(&host) {
    BaselineUpdater::update_first_seen(&mut baseline, &event);
    BaselineUpdater::update_rates(&mut baseline, &event, "metric", rate);
    // ...
}

// Every 5 minutes
baseline_store.save_all();
```

### Phase 3: Wire Windows Signal Engine ⏳ Later
```rust
// In Windows signal engine (or scoring layer)
if let Some(baseline) = baseline_store.get(&host) {
    let (is_new, count) = BaselineQuery::is_new_remote_ip(baseline, exe, ip);
    let (is_burst, threshold) = BaselineQuery::is_burst(baseline, metric, rate, 2.0, 1.0);
    
    // Use in signal logic
    if is_burst {
        signal.weight = Signal::High;
    }
}
```

### Phase 4: Add macOS/Linux Signal Engines ⏳ Future
Each OS builds detectors, consults same baseline.

## Trade-offs

### What We Get
✅ One baseline for all OSes  
✅ Minimal learning period (first-seen is instant)  
✅ Shared incident policy (freeze all at once)  
✅ 3x less code  
✅ Unified testing  

### What We Don't Get (Yet)
⏳ Per-hour×weekday seasonality (EMA is close enough)  
⏳ Covariance matrix (simplified to per-feature variance)  
⏳ Per-identity×process×resource granularity (we have exe + identity, good start)  

### Can Add Later
These don't require rewriting baseline:
- Add `ts_bucket: Option<String>` to queries (hour×weekday)
- Expand FeatureVectorBaseline to store covariance matrix
- Add context-specific baselines (identity + host + time-of-week)

## Why This Works at Scale

**For the first week:**
- Windows has 3+ detector implementations (4624, 5140, Sysmon)
- All feed same baseline
- No macOS/Linux plumbing yet
- Baseline is learning from day 1 (EMA + first-seen)

**For week 2-3:**
- macOS detector added
- Consults same baselines (keyed by host)
- No changes to baseline system
- Baseline now richer (more events, more hosts)

**For week 4+:**
- Linux detector added
- Same story
- One baseline system supporting 3 OSes
- Incident policy is centralized

**Scaling beyond 3 OSes:**
- New OS capture → emit forensic_hooks::Event
- New OS signal engine → query baseline
- Baseline system unchanged

This is why shipping "Windows/Linux/macOS in one week" is possible with shared baselines—you're not building 3 separate learning systems.
