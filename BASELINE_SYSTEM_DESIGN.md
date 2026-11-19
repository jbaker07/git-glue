# Shared Baseline System Design

## Overview

The baseline system is **shared across all operating systems** (Windows, macOS, Linux). Each OS capture binary emits canonical `forensic_hooks::Event`, and a unified baseline engine in `locald/` maintains per-host state.

## Why Shared Baselines?

Instead of building:
- Windows baseliner
- macOS baseliner
- Linux baseliner

We build **one baseline engine** that:
1. Consumes canonical `core::Event` from any OS
2. Maintains per-host state keyed by (host, exe_key, identity_key)
3. Is queried by OS-specific detectors (Windows signal engine, future macOS/Linux engines)

This reduces code duplication and ensures consistent anomaly detection across platforms.

## Baseline Types

### 1. First-Seen Baselines (Allowlist-style, Fast ROI)

Track novel occurrences without requiring "48 hours of learning":

- **Remote IPs per exe**: `{exe_key → set(remote_ip)}`
  - Detect unusual outbound connections
  - Query: "Is this IP new for this exe?"

- **Parent→child exe relationships**: `{parent_exe → set(child_exe)}`
  - Detect anomalous process chains
  - Query: "Is this parent→child relationship new?"

- **File path prefixes per exe**: `{exe_key → set(prefix)}`
  - Detect writing to unusual directories
  - Query: "Does this exe normally write to this path?"

- **Identity activity tags**: `{identity_key → set(tags)}`
  - Track what activities a user typically performs
  - Query: "Has this user ever done this type of activity?"

### 2. Rate Baselines (Burst Detection)

Use simple exponential moving average (EMA) + variance for burst detection:

Per metric (e.g., "admin_share_access:per_host", "connections:per_exe"):
- **EMA**: smoothed average rate
- **EMA_VAR**: smoothed variance
- **Threshold**: `ema + k * sqrt(var)` (typically k=2.0-3.0)

Example:
```
Normal: 2-3 admin share accesses per minute
Baseline ema=2.5, ema_var=0.1
Spike: 50 accesses in one minute
Threshold = 2.5 + 2.0*sqrt(0.1) ≈ 3.1
50 > 3.1 → BURST
```

### 3. Distribution Baselines (Mahalanobis-style)

Track feature vectors for a few high-value metrics:

- **Network feature vector**: (connection_rate, unique_dests, bytes_est, dns_count)
- **Auth feature vector**: (failed_logins, successful_logins, remote_sources, new_ips)
- **Process chain feature vector**: (new_parent_child_count, unsigned_exe_count, writable_dir_launches)

Compute Mahalanobis distance to detect outlier combinations (e.g., "high rate + many new IPs" together is more suspicious than either alone).

## Detection Policy

**Tier-0 Invariants** (alert immediately, no baseline required):
- Injection primitives (CreateRemoteThread + follow-on)
- LSASS access with sensitive rights
- Write→Execute near exec (Linux)
- Tamper attempts against agent
- Log wipe + audit disable chain

**Tier-2/Tier-3** (baseline-weighted, candidate → escalate with corroboration):
- Admin share access burst
- Unusual LOLBin use
- New remote IP for exe (if already have many IPs, don't spam)
- Anomalous process chain
- Feature vector anomaly

Baselines are used to **weight ambiguity**, not to **gate detections**.

## OS-Specific Baseline Inputs

### Windows
Baselines matter most:
- **5140 share access**: rate + unique source IPs + share names
- **4624 remote logon**: logon types, source IPs, usernames
- **Sysmon 3 network**: remote destinations + rates per exe
- **Sysmon 1 process tree**: parent→child norms
- **WMI activity**: which hosts have legitimate WMI noise

### macOS
Baselines matter most:
- **Exec from user-writable paths**: Downloads, /tmp per host/user
- **Sensitive file reads**: ssh keys, keychain artifacts per process
- **launchd plist writes + launchctl patterns**
- **Remote IPs per exe** (like Windows)
- **DYLD injection patterns** for high-churn legit apps

### Linux
Baselines matter most:
- **Outbound connections**: per exe / container
- **Unusual child chains**: shell → curl → bash, etc.
- **Persistence write patterns**: cron, systemd dirs
- **eBPF primitives** (if enabled): W→X frequency, exec-memfd patterns
- **DNS query rate**: per process

## Architecture

```
Baseline Store (in-memory + periodic JSON persistence)
  └─ Per-host baselines
     ├─ FirstSeenBaseline
     │  ├─ remote_ips_per_exe
     │  ├─ parent_exes
     │  ├─ child_exes
     │  ├─ file_prefixes_per_exe
     │  └─ activity_tags_per_identity
     ├─ RateBaselines (HashMap<metric_name, RateBaseline>)
     │  ├─ ema, ema_var, sample_count
     │  └─ per metric: "admin_share_access:per_host", "connections:per_exe:exe_key"
     └─ FeatureVectorBaselines (HashMap<vector_type, FeatureVectorBaseline>)
        ├─ means, variances
        └─ per type: "network", "auth", "process_chain"

BaselineUpdater
  ├─ update_first_seen(baseline, event) → extract from canonical Event
  ├─ update_rates(baseline, metric_name, rate) → update EMA
  └─ update_feature_vector(baseline, vector_type, features) → Welford online

BaselineQuery
  ├─ is_new_remote_ip(baseline, exe, ip) → (bool, count)
  ├─ is_new_parent_child(baseline, parent, child) → bool
  ├─ is_new_identity_activity(baseline, identity, tag) → (bool, count)
  ├─ is_burst(baseline, metric, rate, k, floor) → (bool, threshold)
  └─ is_feature_vector_anomalous(baseline, type, features, threshold) → (distance, bool)

Per-Host Baseline State
  └─ JSON persistence: telemetry_root/baselines/baselines_<host>.json
```

## Usage in Detectors

### Example: Admin Share Burst

```rust
// In Windows signal engine
if event.tags.contains("admin_share_access") {
    let rate = calculate_rate_for_minute();
    let (is_burst, threshold) = BaselineQuery::is_burst(
        baseline, 
        "admin_share_access:per_host",
        rate,
        2.0,  // k=2 std deviations
        1.0   // min_floor=1 access per minute
    );
    
    if is_burst {
        // Trigger AdminShareBurst signal with threshold context
    }
}
```

### Example: New Remote IP for Exe

```rust
// In Windows signal engine
if event.tags.contains("network") {
    let (is_new, total_ips) = BaselineQuery::is_new_remote_ip(
        baseline,
        &exe_key,
        &remote_ip
    );
    
    // Don't alert if exe already connects to many IPs (false positive)
    if is_new && total_ips < 50 {
        // Trigger NewRemoteIPForExe signal
    }
}
```

### Example: Feature Vector Anomaly

```rust
// In cross-platform detector
let mut features = HashMap::new();
features.insert("connection_rate".to_string(), rate);
features.insert("unique_dests".to_string(), unique_ips as f64);

let (distance, is_anomalous) = BaselineQuery::is_feature_vector_anomalous(
    baseline,
    "network",
    &features,
    3.0  // threshold for Mahalanobis distance
);

if is_anomalous {
    // Trigger NetworkFeatureVectorAnomaly
}
```

## Persistence

### Storage Location
```
telemetry_root/baselines/
  ├─ baselines_host1.json
  ├─ baselines_host2.json
  └─ ...
```

### Persistence Policy
- Save every N minutes (e.g., 5)
- Save on graceful shutdown
- Save on "incident start" (snapshot)
- Load on daemon startup

### Example Baseline JSON
```json
{
  "host": "workstation-01",
  "first_seen": {
    "remote_ips_per_exe": {
      "c:\\windows\\system32\\powershell.exe": ["192.168.1.1", "10.0.0.5"]
    },
    "parent_exes": {
      "explorer.exe": ["c:\\program files\\app\\launcher.exe"]
    },
    ...
  },
  "rate_baselines": {
    "admin_share_access:per_host": {
      "ema": 2.5,
      "ema_var": 0.1,
      "sample_count": 50
    }
  },
  "feature_vector_baselines": {
    "network": {
      "means": {"connection_rate": 3.2, "unique_dests": 2.1},
      "variances": {"connection_rate": 0.5, "unique_dests": 0.3},
      "sample_count": 100
    }
  },
  "last_updated_ts": 1703093400000
}
```

## Freezing Baselines

Learning is frozen:
1. **During active incident**: don't update baselines while threat is being mitigated
2. **When sensor capability degrades**: if collection becomes unreliable
3. **When anomaly is persistent**: if threshold is being exceeded consistently

This prevents "normalizing the attack" into the baseline.

## Testing

All modules include comprehensive unit tests:

```bash
cargo test --lib locald::baseline
```

Tests cover:
- FirstSeenBaseline set operations
- RateBaseline EMA calculation and burst detection
- FeatureVectorBaseline Welford online updates
- BaselineStore load/save persistence
- BaselineUpdater event extraction
- BaselineQuery decision functions
