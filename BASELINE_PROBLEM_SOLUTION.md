# The Problem We Solved: Baseline Architecture

## The Original Challenge

You're building an EDR that needs to work on **Windows, macOS, and Linux** without drowning in platform-specific code complexity. The naive approach leads to explosive code growth:

```
Windows Team → Windows detector → Windows baseliner
macOS Team → macOS detector → macOS baseliner
Linux Team → Linux detector → Linux baseliner

Result: 3 separate signal paths, 3 separate baseline systems,
        3 separate persistence policies, 3 separate learning curves
```

By week 2, you have:
- Platform divergence (different features, different learning times)
- Code duplication (baseline logic exists 3 times)
- Incident response chaos (freeze which one? save which one?)
- No cross-platform insights (Windows baseline doesn't help with macOS signal)

## The Solution: One Shared Baseline in locald/

### Design Principle

**All capture binaries emit the same canonical event type** (`forensic_hooks::Event`), and **locald maintains one baseline system** that consumes these events regardless of OS.

```
Windows Capture           macOS Capture             Linux Capture
(parses 4624,            (parses FSEvents,         (parses eBPF,
 Sysmon, WMI)           launchd, system logs)      syscalls)
    ↓                       ↓                          ↓
  forensic_hooks::Event  forensic_hooks::Event   forensic_hooks::Event
    ↓                       ↓                          ↓
    └───────────────────────┼───────────────────────┘
                            ↓
             Shared Baseline System (locald/)
                 (consumes canonical Event)
                            ↓
        BaselineStore (per-host state)
                            ↓
    ┌──────────────────────┼──────────────────────┐
    ↓                      ↓                       ↓
 Windows Signal        macOS Signal          Linux Signal
 Engine                Engine                Engine
 (OS-specific          (OS-specific          (OS-specific
  detectors)            detectors)            detectors)
    ↓                     ↓                      ↓
 Consults baseline    Consults baseline   Consults baseline
```

**Key insight:** Baseline consumption is **OS-agnostic**. All detectors ask the same questions:
- "Is this IP new for this exe?"
- "Is this rate a burst?"
- "Is this feature vector anomalous?"

The answers come from **one shared baseline** keyed by (host, exe/identity, metric).

## How It Works

### Week 1: Windows

```
Windows Capture             Windows Signal Engine
   ↓                              ↓
Event: {                    Detector: AdminShareAccessBurst
  event_id: 5140,              → is_burst("admin_share_access:per_host", rate)?
  share_name: "admin$",         → answer from baseline
  source_ip: "10.0.0.1"
}
   ↓
Baseline update:            Detector: NewRemoteIPForExe
BaselineUpdater::update_first_seen()    → is_new_remote_ip(exe, ip)?
BaselineUpdater::update_rates(...)      → answer from baseline
   ↓
Baseline state:
- remote_ips_per_exe: growing
- rate_baselines["admin_share_access:per_host"]: learning EMA
```

Result: **Windows signal engine gets baseline-aware detectors on day 1.**

### Week 2: Add macOS

```
macOS Capture               Shared Baseline (unchanged!)
   ↓                              ↓
Event: {                    Same BaselineUpdater
  process: "System",        Same BaselineQuery API
  event_type: "launchd_plist_write",    Same per-host storage
  path: "/Library/LaunchDaemons/..."
}
   ↓
Baseline update:            macOS Signal Engine
Same BaselineUpdater::update_first_seen()    (builds on baseline)
   ↓
Baseline state:
- Now contains data from Windows + macOS (same host)
- rate_baselines: learning from all event types
```

Result: **macOS detector gets baseline immediately without rewriting anything.**

### Week 3: Add Linux

```
Linux Capture               Shared Baseline (still unchanged!)
   ↓                              ↓
Event: {                    Same BaselineUpdater
  syscall: "execve",        Same BaselineQuery API
  path: "/usr/bin/curl",    Same per-host storage
  child_pid: 1234
}
   ↓
Baseline update:            Linux Signal Engine
Same BaselineUpdater        (builds on baseline)
   ↓
Baseline state:
- Now contains data from Windows + macOS + Linux
- rate_baselines: learning from all three OSes
```

Result: **Linux detector gets months of baseline data from week 1-2 for free.**

## Why This Is Faster Than Per-OS Approach

### Per-OS Cost Analysis

| Task | Windows Only | Windows + macOS | Windows + macOS + Linux |
|------|----------|-----------|-----------|
| Baseline logic | 1x (150 lines) | 3x (450 lines) | 3x (450 lines) |
| Baseline tests | 1x (50 tests) | 3x (150 tests) | 3x (150 tests) |
| Persistence | 1x (30 lines) | 3x (90 lines) | 3x (90 lines) |
| Incident freeze | 1x | 3x (sync?) | 3x (sync?) |
| Bug fixes | 1x fix | 3x (backport to all?) | 3x |
| New feature | 1x implementation | 3x implementations | 3x implementations |
| **Total code** | ~250 lines | ~750 lines | ~750 lines |

### Shared Baseline Cost Analysis

| Task | Windows Only | Windows + macOS | Windows + macOS + Linux |
|------|----------|-----------|-----------|
| Baseline logic | 1x (150 lines) | 1x (unchanged) | 1x (unchanged) |
| Baseline tests | 1x (50 tests) | 1x (unchanged) | 1x (unchanged) |
| Persistence | 1x (30 lines) | 1x (unchanged) | 1x (unchanged) |
| Incident freeze | 1x | 1x (unchanged) | 1x (unchanged) |
| Bug fixes | 1x fix | 1x fix (global) | 1x fix (global) |
| New feature | 1x implementation | 1x implementation (global) | 1x implementation (global) |
| OS-specific detectors | ~400 lines (Windows) | +~300 lines (macOS) | +~300 lines (Linux) |
| **Total code** | ~600 lines | ~900 lines | ~1200 lines |

**Key difference:** Baseline logic doesn't grow; only OS-specific detectors grow.

With per-OS approach: Baseline code **triples** as you add OSes.  
With shared approach: Baseline code **stays constant** as you add OSes.

## Concrete Example: Detecting Anomalous Process Chain

### Per-OS Approach

**Windows Baseliner:**
```rust
fn update_process_chain_windows(baseline: &mut WindowsBaseline, event: &WindowsEvent) {
    if event.event_id == 1 { // Sysmon process create
        let parent = extract_parent_process_windows(event);
        let child = extract_child_process_windows(event);
        baseline.parent_child_chains.entry(parent)
            .or_insert_with(HashSet::new)
            .insert(child);
    }
}

fn is_anomalous_chain_windows(baseline: &WindowsBaseline, parent: &str, child: &str) -> bool {
    !baseline.parent_child_chains.get(parent)
        .map(|children| children.contains(child))
        .unwrap_or(false)
}
```

**macOS Baseliner (copy-paste):**
```rust
fn update_process_chain_macos(baseline: &mut MacOSBaseline, event: &MacOSEvent) {
    if event.event_type == "process_exec" { // FSEvents
        let parent = extract_parent_process_macos(event);
        let child = extract_child_process_macos(event);
        baseline.parent_child_chains.entry(parent)
            .or_insert_with(HashSet::new)
            .insert(child);
    }
}

fn is_anomalous_chain_macos(baseline: &MacOSBaseline, parent: &str, child: &str) -> bool {
    !baseline.parent_child_chains.get(parent)
        .map(|children| children.contains(child))
        .unwrap_or(false)
}
```

**Linux Baseliner (copy-paste again):**
```rust
fn update_process_chain_linux(baseline: &mut LinuxBaseline, event: &LinuxEvent) {
    if event.syscall == "execve" { // eBPF
        let parent = extract_parent_process_linux(event);
        let child = extract_child_process_linux(event);
        baseline.parent_child_chains.entry(parent)
            .or_insert_with(HashSet::new)
            .insert(child);
    }
}

fn is_anomalous_chain_linux(baseline: &LinuxBaseline, parent: &str, child: &str) -> bool {
    !baseline.parent_child_chains.get(parent)
        .map(|children| children.contains(child))
        .unwrap_or(false)
}
```

**Problem:** Same logic, written 3 times. Bug in Windows version → need to fix macOS/Linux.

### Shared Baseline Approach

**Event Parsing (OS-specific, happens once per OS):**
```rust
// Windows capture
Event {
    parent_exe: "explorer.exe",
    child_exe: "cmd.exe",
    ...
}

// macOS capture
Event {
    parent_exe: "/bin/bash",
    child_exe: "/usr/bin/curl",
    ...
}

// Linux capture
Event {
    parent_exe: "/usr/bin/python",
    child_exe: "/usr/bin/pip",
    ...
}
```

**Baseline Update (shared, happens once total):**
```rust
fn update_first_seen(baseline: &mut HostBaseline, event: &Event) {
    if let (Some(parent), Some(child)) = (
        event.fields.get("parent_exe"),
        event.fields.get("child_exe")
    ) {
        baseline.first_seen.parent_exes
            .entry(parent.clone())
            .or_insert_with(HashSet::new)
            .insert(child.clone());
    }
}
```

**Baseline Query (shared, happens once total):**
```rust
fn is_new_parent_child(baseline: &HostBaseline, parent: &str, child: &str) -> bool {
    baseline.first_seen.parent_exes
        .get(parent)
        .map(|children| !children.contains(child))
        .unwrap_or(true)
}
```

**Result:** One implementation that works for all three OSes.

## Timeline Impact

### Week 1: Windows Only
- Day 1-2: Implement Windows capture
- Day 2-3: Implement Windows signal engine
- Day 3: Implement baseline system
- Day 4-5: Integration + testing
- **Result:** Baseline-aware Windows detectors

### Week 2: Add macOS
- Day 1-2: Implement macOS capture
- Day 2-3: Implement macOS signal engine + detectors
- **No baseline work** (reuse existing system)
- Day 3-4: Integration + testing
- **Result:** Windows + macOS both using shared baseline

### Week 3: Add Linux
- Day 1-2: Implement Linux capture
- Day 2-3: Implement Linux signal engine + detectors
- **No baseline work** (reuse existing system)
- Day 3-4: Integration + testing
- **Result:** Windows + macOS + Linux all using shared baseline

**Total time: 3 weeks, shipping all three OSes.**

If each OS had its own baseliner:
- Week 2 would require rewriting baseline + integration
- Week 3 would require rewriting baseline again
- Total time: ~4-5 weeks

## What the Shared System Actually Does

The baseline system (`locald/baseline/`) provides:

1. **FirstSeenBaseline** (zero learning period)
   - Remembers: "This exe has connected to these IPs"
   - Remembers: "This process chain has occurred before"
   - Remembers: "This user has performed this activity type"
   - Fast ROI: alerts can fire on day 1

2. **RateBaseline** (gradual learning via EMA)
   - Learns: "Admin shares accessed ~2x per minute on average"
   - Learns: "Network connections spike at this rate"
   - Adapts: EMA smoothing means no "learning period warmup"

3. **FeatureVectorBaseline** (correlative anomalies)
   - Learns: "High connection rate + many new IPs together is suspicious"
   - Learns: "Failed logins + successful logins in unusual ratio"
   - Detects: Combinations that would miss individual thresholds

## Key Takeaway

**Shared baseline = Capability at launch, not at week 4.**

- Week 1 Windows gets baseline queries
- Week 2 macOS gets baseline queries (immediately)
- Week 3 Linux gets baseline queries (immediately)
- No rewrite, no backport, no "catch up"

This is why the architecture is called "one baseliner" not "one-of-three baseliner".
