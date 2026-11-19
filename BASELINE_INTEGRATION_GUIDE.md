# Baseline System: Integration Quick Reference

## Module Structure

```
locald/baseline/
├── mod.rs                    # Exports: HostBaseline, BaselineStore, BaselineUpdater, BaselineQuery
├── types.rs                  # Data structures: FirstSeenBaseline, RateBaseline, FeatureVectorBaseline, HostBaseline
├── baseline_store.rs         # Persistence: load/save per-host JSON
├── baseline_update.rs        # Event processor: update baselines from canonical Event
└── baseline_query.rs         # Query API: novelty, burst, anomaly checks
```

## Quick API Reference

### BaselineStore (Persistence)

```rust
use locald::baseline::{BaselineStore};

// Create and load
let mut store = BaselineStore::new(telemetry_root.join("baselines"));
store.load_all()?;

// Get baseline for a host
let baseline = store.get_or_create("workstation-01")?;

// Update (consume mutable reference)
let mut baseline = store.get_or_create("workstation-01")?;
BaselineUpdater::update_first_seen(&mut baseline, &event);

// Save
store.save("workstation-01")?;
store.save_all()?;  // Persist all hosts
```

### BaselineUpdater (Event Processing)

```rust
use locald::baseline::BaselineUpdater;

// After parsing event from capture binary
if let Some(mut baseline) = store.get_or_create(&host) {
    // Extract and store novel items
    BaselineUpdater::update_first_seen(&mut baseline, &event);
    
    // Update rate baselines (e.g., connections per minute)
    BaselineUpdater::update_rates(
        &mut baseline,
        &event,
        "admin_share_access:per_host",  // metric name
        5.0  // observed rate
    );
    
    // Update feature vectors
    let mut features = std::collections::HashMap::new();
    features.insert("connection_rate".to_string(), 3.2);
    features.insert("unique_dests".to_string(), 2.0);
    BaselineUpdater::update_feature_vector(
        &mut baseline,
        "network",  // vector type: "network", "auth", "process_chain"
        &features
    );
}
```

### BaselineQuery (Reading Baselines for Detection)

#### Query 1: Is Remote IP New for This Exe?

```rust
use locald::baseline::BaselineQuery;

let (is_new, total_count) = BaselineQuery::is_new_remote_ip(
    &baseline,
    "c:\\windows\\system32\\powershell.exe",  // exe_key
    "192.168.1.50"  // remote_ip
);

// Returns: (bool, usize) = (is_new, total_ips_seen_for_exe)
// Use: Don't alert if exe already connects to 100+ IPs (false positive prevention)
if is_new && total_count < 50 {
    // Potential signal: NewRemoteIPForExe
}
```

#### Query 2: Is Parent→Child Process Relationship New?

```rust
let is_new = BaselineQuery::is_new_parent_child(
    &baseline,
    "explorer.exe",      // parent_exe
    "cmd.exe"            // child_exe
);

// Returns: bool
// Use: Detect anomalous process chains
if is_new {
    // Potential signal: AnomalousProcessChain
}
```

#### Query 3: Is Activity Tag New for This Identity?

```rust
let (is_new_tag, total_tags) = BaselineQuery::is_new_identity_activity(
    &baseline,
    "user@domain.com",   // identity_key
    "lolbin_usage"       // activity tag
);

// Returns: (bool, usize) = (is_new_tag, total_distinct_tags)
// Use: Track novel activities per user (e.g., first time using certutil, first LOLBin)
if is_new_tag {
    // Potential signal: NewIdentityActivity
}
```

#### Query 4: Is File Path Prefix New for This Exe?

```rust
let is_new = BaselineQuery::is_new_file_prefix(
    &baseline,
    "c:\\program files\\app\\myapp.exe",  // exe_key
    "/var/tmp"  // path prefix (first 2 segments)
);

// Returns: bool
// Use: Detect exe writing to unusual directories
if is_new {
    // Potential signal: ExeWriteToUnusualPath
}
```

#### Query 5: Is This Rate a Burst?

```rust
let (is_burst, threshold) = BaselineQuery::is_burst(
    &baseline,
    "admin_share_access:per_host",  // metric name (any string)
    25.0,   // observed rate (shares per minute)
    2.0,    // threshold_k (standard deviations, typically 2.0-3.0)
    1.0     // min_floor (minimum threshold floor)
);

// Returns: (bool, f64) = (is_burst, computed_threshold)
// Formula: threshold = max(ema + k*sqrt(var), min_floor)
// Use: Detect burst activity (spike detection)
if is_burst {
    // Potential signal: AdminShareAccessBurst
    // Include threshold in signal for context
}
```

#### Query 6: Is Feature Vector Anomalous?

```rust
let mut features = std::collections::HashMap::new();
features.insert("connection_rate".to_string(), 3.2);
features.insert("unique_dests".to_string(), 2.0);

let (distance, is_anomalous) = BaselineQuery::is_feature_vector_anomalous(
    &baseline,
    "network",          // vector type
    &features,
    3.0                 // threshold (Mahalanobis distance)
);

// Returns: (f64, bool) = (distance, is_anomalous)
// Use: Detect when combination of features is unusual
if is_anomalous {
    // Potential signal: FeatureVectorAnomaly
    // Include distance in signal for severity
}
```

#### Debug Queries: Get Current Baseline State

```rust
// Get EMA for a metric
if let Some(ema) = BaselineQuery::get_ema(&baseline, "admin_share_access:per_host") {
    println!("Current EMA for metric: {}", ema);
}

// Get statistics for a feature vector
if let Some((mean_mean, mean_var, sample_count)) = 
    BaselineQuery::get_feature_vector_stats(&baseline, "network") {
    println!("Network vector trained on {} samples", sample_count);
}
```

## Common Patterns

### Pattern 1: Monitor Admin Share Access

```rust
// In Windows signal detector
fn check_admin_share_access(event: &forensic_hooks::Event, baseline: &HostBaseline) {
    if event.fields.get("event_type").map(|e| e.as_str()) == Some("admin_share_access") {
        if let Some(count) = extract_rate(&event) {  // shares accessed per minute
            let (is_burst, threshold) = BaselineQuery::is_burst(
                baseline,
                "admin_share_access:per_host",
                count as f64,
                2.5,  // higher threshold = fewer false positives
                0.5   // minimum: at least this rate to flag
            );
            
            if is_burst {
                emit_signal(AdminShareAccessBurst {
                    rate: count,
                    threshold,
                    weight: High,
                });
            }
        }
    }
}
```

### Pattern 2: Detect Novel Remote Connections

```rust
// In Windows signal detector
fn check_remote_connection(event: &forensic_hooks::Event, baseline: &HostBaseline) {
    if event.fields.get("event_type").map(|e| e.as_str()) == Some("network_connect") {
        if let (Some(exe), Some(dest_ip)) = (
            event.fields.get("src_exe"),
            event.fields.get("dest_ip")
        ) {
            let (is_new, count) = BaselineQuery::is_new_remote_ip(
                baseline,
                exe,
                dest_ip
            );
            
            // Only alert on novel IPs if exe doesn't already connect to tons of IPs
            if is_new && count < 50 {
                emit_signal(NewRemoteConnectionForExe {
                    exe,
                    dest_ip,
                    prior_destinations: count,
                    weight: Medium,  // low certainty—many exes are chatty
                });
            }
        }
    }
}
```

### Pattern 3: Detect Anomalous Execution Patterns

```rust
// In cross-platform detector
fn check_execution_pattern(event: &forensic_hooks::Event, baseline: &HostBaseline) {
    if event.fields.get("event_type").map(|e| e.as_str()) == Some("process_create") {
        let mut features = std::collections::HashMap::new();
        
        // Extract multiple indicators
        if let Some(v) = extract_unique_child_count(&event) {
            features.insert("unique_children".to_string(), v);
        }
        if let Some(v) = extract_spawn_rate(&event) {
            features.insert("spawn_rate".to_string(), v);
        }
        if let Some(v) = extract_unsigned_exe_count(&event) {
            features.insert("unsigned_count".to_string(), v);
        }
        
        let (distance, is_anomalous) = BaselineQuery::is_feature_vector_anomalous(
            baseline,
            "process_chain",
            &features,
            3.0  // tight threshold for process anomaly
        );
        
        if is_anomalous {
            emit_signal(ProcessChainAnomaly {
                mahalanobis_distance: distance,
                features,
                weight: Medium,  // context matters
            });
        }
    }
}
```

## Integration Checklist

- [ ] Create BaselineStore in edr_locald main()
- [ ] Load all baselines on startup: `store.load_all()`
- [ ] Update baselines in ingest_segment() for each event
- [ ] Persist baselines every 5 minutes: `store.save_all()`
- [ ] Persist baselines on shutdown
- [ ] Freeze baselines on incident start: `baseline_frozen = true`
- [ ] Wire Windows signal engine queries (is_new_remote_ip, is_burst, etc.)
- [ ] Add feature extraction to Windows signal detector
- [ ] Test: Verify baselines persist across restarts
- [ ] Test: Verify queries return sensible values

## JSON Persistence Format

Baselines are stored in:
```
telemetry_root/baselines/baselines_<hostname>.json
```

Example:
```json
{
  "host": "workstation-01",
  "first_seen": {
    "remote_ips_per_exe": {
      "c:\\windows\\system32\\powershell.exe": ["192.168.1.1", "10.0.0.5"],
      "c:\\program files\\chrome\\chrome.exe": ["142.251.41.4"]
    },
    "parent_exes": { "explorer.exe": ["c:\\program files\\launcher.exe"] },
    "child_exes": { "powershell.exe": ["cmd.exe", "notepad.exe"] },
    "file_prefixes_per_exe": {
      "c:\\windows\\system32\\powershell.exe": ["c:\\users\\admin", "/tmp"]
    },
    "exes_per_identity": {
      "admin@corp.com": ["powershell.exe", "notepad.exe"]
    },
    "activity_tags_per_identity": {
      "admin@corp.com": ["lolbin_usage", "admin_share", "remote_exec"]
    }
  },
  "rate_baselines": {
    "admin_share_access:per_host": {
      "ema": 2.3,
      "ema_var": 0.5,
      "last_ts_min": 1703093400000,
      "sample_count": 45
    },
    "connections:per_minute": {
      "ema": 12.5,
      "ema_var": 4.2,
      "last_ts_min": 1703093400000,
      "sample_count": 120
    }
  },
  "feature_vector_baselines": {
    "network": {
      "means": {
        "connection_rate": 3.2,
        "unique_dests": 2.1,
        "bytes_est": 102400.0
      },
      "variances": {
        "connection_rate": 0.8,
        "unique_dests": 0.3,
        "bytes_est": 50000000.0
      },
      "sample_count": 150
    }
  },
  "last_updated_ts": 1703093400000
}
```

## Testing

All baseline modules include unit tests:

```bash
# Run all baseline tests
cargo test --lib locald::baseline

# Run specific test
cargo test --lib locald::baseline::types::test_rate_baseline_ema

# Run with output
cargo test --lib locald::baseline -- --nocapture
```

Key test scenarios:
- FirstSeenBaseline: Set insertion and membership
- RateBaseline: EMA calculation with known values, burst detection threshold
- FeatureVectorBaseline: Online Welford mean/variance, distance computation
- BaselineStore: JSON round-trip persistence
- BaselineUpdater: Event field extraction
- BaselineQuery: Novelty and burst decision logic
