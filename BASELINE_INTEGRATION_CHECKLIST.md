# Baseline System: Integration Checklist

## Phase 1: Verification (Immediate)

- [ ] **Code compiles without errors**
  ```bash
  cd /Users/jermainebaker/git-glue
  cargo build --bin edr_locald
  ```
  Expected: No compilation errors

- [ ] **All unit tests pass**
  ```bash
  cargo test --lib locald::baseline
  ```
  Expected: 12 passed

- [ ] **Module exports are correct**
  ```bash
  grep -n "pub mod baseline" locald/mod.rs
  ```
  Expected: Module visible

- [ ] **Files exist in correct location**
  ```bash
  ls -la locald/baseline/
  ```
  Expected: types.rs, baseline_store.rs, baseline_update.rs, baseline_query.rs, mod.rs

## Phase 2: edr_locald Integration (Week 1)

### 2.1 Initialize Baseline Store in main()

**Location:** `edr_locald/main.rs` (or wherever edr_locald initializes)

**Add imports:**
```rust
use locald::baseline::{BaselineStore, BaselineUpdater};
```

**Initialize:**
```rust
// In main(), after signal state creation
let telemetry_root = PathBuf::from(&config.telemetry_root);
let mut baseline_store = BaselineStore::new(telemetry_root.join("baselines"));
baseline_store.load_all().map_err(|e| {
    eprintln!("Failed to load baselines: {}", e);
    // Can continue without baselines (graceful degradation)
})?;
```

**Checklist:**
- [ ] Create BaselineStore with correct path
- [ ] Call load_all() on startup
- [ ] Handle error gracefully (log but don't crash)
- [ ] Pass baseline_store to event loop

### 2.2 Update Baselines in ingest_segment()

**Location:** `edr_locald/ingest_segment()` or event processing loop

**For each canonical event:**
```rust
// After event is parsed/constructed
if let Some(host) = &event.fields.get("host") {
    if let Ok(mut baseline) = baseline_store.get_or_create(host) {
        // Update first-seen sets (always)
        BaselineUpdater::update_first_seen(&mut baseline, &event);
        
        // Update rate baselines (for events with rates)
        if let Some(event_type) = event.fields.get("event_type") {
            match event_type.as_str() {
                "admin_share_access" => {
                    let rate = calculate_admin_shares_per_minute(&event);
                    BaselineUpdater::update_rates(&mut baseline, &event, 
                        "admin_share_access:per_host", rate);
                }
                "network_connect" => {
                    let rate = calculate_connections_per_minute(&event);
                    BaselineUpdater::update_rates(&mut baseline, &event,
                        "connections:per_minute", rate);
                }
                _ => {}
            }
        }
        
        // Update feature vectors (periodic, not per-event)
        if should_update_feature_vector(&event) {
            let mut features = extract_network_features(&event);
            BaselineUpdater::update_feature_vector(&mut baseline, "network", &features);
        }
    }
}
```

**Checklist:**
- [ ] Get baseline from store (get_or_create)
- [ ] Call update_first_seen() for all events
- [ ] Call update_rates() for rate-relevant events
- [ ] Call update_feature_vector() periodically or on key event types
- [ ] Handle errors gracefully

### 2.3 Persist Baselines

**Location:** Background thread or shutdown handler

**Option A: Periodic (every 5 minutes)**
```rust
// In a background thread
loop {
    std::thread::sleep(Duration::from_secs(300));  // 5 minutes
    if let Err(e) = baseline_store.save_all() {
        eprintln!("Failed to save baselines: {}", e);
    }
}
```

**Option B: On Shutdown**
```rust
// In graceful shutdown handler
baseline_store.save_all().map_err(|e| {
    eprintln!("Failed to save baselines on shutdown: {}", e);
})?;
```

**Checklist:**
- [ ] Persistence runs periodically (5-min) or on shutdown
- [ ] Errors are logged but don't crash
- [ ] Files are created in telemetry_root/baselines/

### 2.4 Verify Baseline Persistence

**Manual test:**
```bash
# Run edr_locald for 10 seconds
timeout 10 cargo run --bin edr_locald

# Check if baseline files were created
ls -la /path/to/telemetry_root/baselines/

# Inspect JSON
cat /path/to/telemetry_root/baselines/baselines_workstation-01.json | jq .
```

**Checklist:**
- [ ] Baseline directory is created
- [ ] JSON files appear for each host
- [ ] JSON contains valid first_seen, rate_baselines, feature_vector_baselines
- [ ] last_updated_ts is recent

## Phase 3: Signal Engine Integration (Week 1-2)

### 3.1 Wire Windows Signal Engine to Baseline

**Location:** Windows signal engine (or scoring layer that consumes Window events)

**Import:**
```rust
use locald::baseline::{BaselineQuery};
```

**Pattern: Check if Remote IP is New**
```rust
fn check_remote_connection(event: &Event, baseline: &HostBaseline) -> Option<Signal> {
    if event.tags.contains("network_connect") {
        if let (Some(src_exe), Some(dest_ip)) = (
            event.fields.get("src_exe"),
            event.fields.get("dest_ip")
        ) {
            let (is_new, prior_count) = BaselineQuery::is_new_remote_ip(
                baseline,
                src_exe,
                dest_ip
            );
            
            // Only alert on novel IPs if exe hasn't connected to tons of places
            if is_new && prior_count < 50 {
                return Some(Signal {
                    signal_type: "NewRemoteIPForExe",
                    severity: Medium,
                    context: format!("Exe {} connected to new IP {} (seen {} before)", 
                        src_exe, dest_ip, prior_count),
                });
            }
        }
    }
    None
}
```

**Checklist:**
- [ ] Windows signal detector has access to baseline
- [ ] is_new_remote_ip query is integrated
- [ ] False positives are reduced (e.g., ignore if exe has 50+ prior IPs)
- [ ] Severity/weight is context-aware

**Pattern: Check if Rate is Burst**
```rust
fn check_admin_share_burst(events: &[Event], baseline: &HostBaseline) -> Option<Signal> {
    let rate = calculate_admin_shares_per_minute(events);
    let (is_burst, threshold) = BaselineQuery::is_burst(
        baseline,
        "admin_share_access:per_host",
        rate,
        2.5,  // k = 2.5 std deviations for admin shares
        0.5   // minimum threshold of 0.5 shares per minute
    );
    
    if is_burst {
        return Some(Signal {
            signal_type: "AdminShareAccessBurst",
            severity: High,
            context: format!("Admin share access rate {} exceeds threshold {}", rate, threshold),
        });
    }
    None
}
```

**Checklist:**
- [ ] Rate baselines are created during normal operation
- [ ] is_burst() returns sensible thresholds (ema + k*sqrt(var))
- [ ] Burst detection fires when rate exceeds threshold
- [ ] False positive rate is acceptable

**Pattern: Check if Process Chain is Anomalous**
```rust
fn check_process_chain_anomaly(event: &Event, baseline: &HostBaseline) -> Option<Signal> {
    if event.tags.contains("process_create") {
        if let (Some(parent), Some(child)) = (
            event.fields.get("parent_exe"),
            event.fields.get("child_exe")
        ) {
            let is_new = BaselineQuery::is_new_parent_child(baseline, parent, child);
            
            if is_new {
                return Some(Signal {
                    signal_type: "AnomalousProcessChain",
                    severity: Medium,
                    context: format!("New process chain: {} → {}", parent, child),
                });
            }
        }
    }
    None
}
```

**Checklist:**
- [ ] Parent-child relationships are baselined
- [ ] is_new_parent_child() query is integrated
- [ ] New chains trigger signals appropriately
- [ ] Whitelisting of expected chains works

### 3.2 Feature Extraction for Rate/Vector Baselines

**Location:** Event preprocessing before signal detectors

**Rate baseline extraction:**
```rust
fn extract_rates(event: &Event) -> Vec<(&'static str, f64)> {
    let mut rates = Vec::new();
    
    if event.tags.contains("admin_share_access") {
        // Extract: shares accessed per minute
        if let Some(count) = event.fields.get("share_count") {
            rates.push(("admin_share_access:per_host", count.parse().unwrap_or(1.0)));
        }
    }
    
    if event.tags.contains("network_connect") {
        // Extract: connections per minute
        rates.push(("connections:per_minute", 1.0));  // 1 per event
    }
    
    rates
}
```

**Feature vector extraction:**
```rust
fn extract_network_features(event: &Event) -> HashMap<String, f64> {
    let mut features = HashMap::new();
    
    // Per event type, extract meaningful features
    if let Some(conn_rate) = calculate_connection_rate(event) {
        features.insert("connection_rate".to_string(), conn_rate);
    }
    
    if let Some(unique_ips) = count_unique_destinations(event) {
        features.insert("unique_dests".to_string(), unique_ips as f64);
    }
    
    if let Some(bytes) = estimate_bytes_transferred(event) {
        features.insert("bytes_est".to_string(), bytes as f64);
    }
    
    features
}
```

**Checklist:**
- [ ] Rate extraction is implemented for key metrics
- [ ] Feature vector extraction covers "network", "auth", "process_chain"
- [ ] Features are normalized (e.g., per-minute, per-host)
- [ ] Extraction is fast (doesn't slow down processing)

### 3.3 Test Baseline Queries

**Integration test:**
```bash
# Create a test event flow
# Event 1: powershell.exe → 192.168.1.1 (NEW)
# Event 2: powershell.exe → 192.168.1.1 (known)
# Event 3: powershell.exe → 10.0.0.1 (NEW)

# Verify:
# - Event 1: is_new_remote_ip() returns (true, 1)
# - Event 2: is_new_remote_ip() returns (false, 2)
# - Event 3: is_new_remote_ip() returns (true, 2)
```

**Checklist:**
- [ ] is_new_remote_ip() works for known/unknown IPs
- [ ] Prior counts match expectations
- [ ] Rate baselines learn and threshold correctly
- [ ] Feature vectors compute distance correctly

## Phase 4: Baseline Freezing & Incident Policy (Week 2)

### 4.1 Freeze Learning During Incidents

**Location:** Incident management code

**Add flag to BaselineStore:**
```rust
pub struct BaselineStore {
    store: HashMap<String, HostBaseline>,
    baselines_dir: PathBuf,
    pub frozen: bool,  // Add this field
}

impl BaselineStore {
    pub fn freeze(&mut self) {
        self.frozen = true;
    }
    
    pub fn unfreeze(&mut self) {
        self.frozen = false;
    }
}
```

**Check freeze before update:**
```rust
// In ingest_segment()
if !baseline_store.frozen {
    BaselineUpdater::update_first_seen(&mut baseline, &event);
    BaselineUpdater::update_rates(&mut baseline, &event, "metric", rate);
}
```

**Call freeze on incident:**
```rust
// When incident is detected
baseline_store.freeze();

// When incident is resolved
baseline_store.unfreeze();
```

**Checklist:**
- [ ] Baseline has freeze/unfreeze methods
- [ ] Updates check frozen flag
- [ ] Incident detection sets frozen flag
- [ ] Incident resolution clears frozen flag

### 4.2 Baseline Snapshot on Incident Start

**Location:** Incident handler

```rust
// On incident start, save current baseline state
baseline_store.save_all().map_err(|e| {
    eprintln!("Failed to snapshot baseline for incident: {}", e);
})?;

// Optionally rename for audit trail
// cp baselines_<host>.json baselines_<host>_incident_<timestamp>.json
```

**Checklist:**
- [ ] Baselines are saved before incident processing
- [ ] Snapshots are labeled with timestamp
- [ ] Snapshots can be compared post-incident

## Phase 5: Multi-OS Support (Week 3+)

### 5.1 macOS Signal Engine (Reuse Baseline)

**No baseline changes needed.** Just add macOS detector:

```rust
// macOS event capture parses FSEvents, system logs, etc.
// Emits canonical Event with macOS-specific fields:
// - event_type: "launchd_plist_write"
// - path: "/Library/LaunchDaemons/..."
// - parent_exe: "/bin/bash"
// - child_exe: "/usr/bin/curl"

// Same BaselineUpdater processes it
BaselineUpdater::update_first_seen(&mut baseline, &event);

// Same BaselineQuery answers questions
let is_new = BaselineQuery::is_new_parent_child(baseline, parent, child);
```

**Checklist:**
- [ ] macOS events emit canonical Event
- [ ] No baseline code changes needed
- [ ] macOS baseline grows from Windows data
- [ ] Detectors query same baseline

### 5.2 Linux Signal Engine (Reuse Baseline)

**Same as macOS:** Just add Linux detector, reuse baseline.

```rust
// Linux event capture parses eBPF syscalls, cron logs, etc.
// Emits canonical Event with Linux-specific fields:
// - event_type: "execve"
// - path: "/usr/bin/python"
// - parent_exe: "/bin/bash"
// - child_exe: "/usr/bin/pip"

// Same BaselineUpdater
BaselineUpdater::update_first_seen(&mut baseline, &event);

// Same BaselineQuery
let is_new = BaselineQuery::is_new_parent_child(baseline, parent, child);
```

**Checklist:**
- [ ] Linux events emit canonical Event
- [ ] No baseline code changes needed
- [ ] Linux baseline grows from Windows + macOS data
- [ ] Detectors query same baseline

## Success Metrics

### Week 1
- [ ] Baselines persist to disk
- [ ] First-seen sets are populated
- [ ] Rate baselines show EMA learning
- [ ] Windows signal engine queries baseline
- [ ] No degradation to event processing latency

### Week 2
- [ ] Admin share burst detection works
- [ ] New remote IP detection works
- [ ] Process chain detection works
- [ ] Baseline freezing works during incidents
- [ ] False positive rate is acceptable (<5%)

### Week 3
- [ ] macOS detector uses same baseline
- [ ] Linux detector uses same baseline
- [ ] Cross-OS correlation possible (e.g., same IP accessed from Windows + macOS)
- [ ] All three OSes shipping with baseline awareness

### End State
- ✅ One shared baseline per host (not per OS)
- ✅ Zero learning period (first-seen instant)
- ✅ Operational visibility (JSON readable)
- ✅ Fast ROI (alerts from day 1)
- ✅ Scalable (no code duplication)

## Troubleshooting

### Problem: Baseline file not created
**Solution:**
```bash
# Check directory exists
mkdir -p /path/to/telemetry_root/baselines

# Check permissions
chmod 755 /path/to/telemetry_root/baselines

# Check logs
journalctl -u edr_locald | grep baseline
```

### Problem: Baseline not learning
**Solution:**
```bash
# Verify events are being processed
grep "event_type" /path/to/telemetry_root/baselines/baselines_*.json | head

# Check if updates are called
# Add logging: println!("Updating baseline for {}", host);
```

### Problem: Rate threshold too high/low
**Solution:**
```bash
# Inspect baseline EMA
cat /path/to/telemetry_root/baselines/baselines_*.json | jq '.rate_baselines'

# Adjust k parameter (default 2.0, try 1.5-3.0)
let (is_burst, threshold) = BaselineQuery::is_burst(baseline, metric, rate, 2.5, 1.0);
```

### Problem: Too many false positives
**Solution:**
1. Check Tier-0 invariants are truly gated (never baseline-weighted)
2. Increase prior count threshold: `if is_new && count < 100`
3. Increase burst k threshold: `2.5` → `3.0`
4. Add identity-based filtering

## Final Verification Checklist

Before declaring baseline system "complete":

- [ ] All 5 baseline modules compile without errors
- [ ] All 12 unit tests pass
- [ ] Baseline initialized in edr_locald main()
- [ ] Baselines updated in ingest_segment() for each event
- [ ] Baselines persisted every 5 minutes + on shutdown
- [ ] Baseline files are JSON and human-readable
- [ ] Windows signal engine queries baseline for ≥3 detector types
- [ ] False positive rate is acceptable
- [ ] Baseline freezing works on incident start
- [ ] macOS detector can reuse same baseline (verified in code, pending full macOS impl)
- [ ] Linux detector can reuse same baseline (verified in code, pending full Linux impl)
- [ ] Operational documentation is complete

## Expected Timeline

| Week | Milestone | Status |
|------|-----------|--------|
| 1 | Baseline infrastructure complete | ✅ Done |
| 1 | edr_locald integration | ⏳ Next |
| 1-2 | Windows signal engine queries | ⏳ Next |
| 2 | Rate/feature baselines learning | ⏳ Next |
| 2 | Incident freezing policy | ⏳ Next |
| 3 | macOS signal engine | ⏳ Future |
| 3 | Linux signal engine | ⏳ Future |

**Total effort:** ~3-4 weeks from start to shipping all three OSes with baseline support.
