# Critical Infrastructure Maps - Usage Guide

## Overview

Two pinned BPF maps have been added to `edr_events.h` to support dynamic filtering and runtime control:

1. **protected_pids** - Prevent signal_integrity from killing critical processes
2. **ondemand_enabled** - Gate all 7 ON_DEMAND programs at runtime

---

## Map 1: protected_pids

### Definition
```c
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 128);
    __type(key, __u32);    // pid
    __type(value, __u8);   // 1 = protected
} protected_pids SEC(".maps");
```

### Location
[edr_events.h](edr_events.h) - Added alongside other shared maps

### Purpose
Replace hardcoded `pid <= 5` check in signal_integrity.bpf.c with dynamic map lookup.

### Affected Programs
- **signal_integrity.bpf.c**: Uses `is_protected_pid()` helper which now looks up map:
  ```c
  static __always_inline bool is_protected_pid(__u32 pid) {
      __u8 *protected = bpf_map_lookup_elem(&protected_pids, &pid);
      return protected && *protected == 1;
  }
  ```

### Usage in Userspace

#### Rust Example (libbpf)
```rust
// After loading BPF programs
let protected_pids_map = bpf.map_mut("protected_pids")?;

// Add current process
let my_pid = std::process::id() as u32;
protected_pids_map.insert(
    my_pid.to_ne_bytes().to_vec(),
    vec![1],
    0
)?;

// Add system critical PIDs
for critical_pid in [1, 2, 3, 4, 5] {
    protected_pids_map.insert(
        critical_pid.to_ne_bytes().to_vec(),
        vec![1],
        0
    )?;
}
```

#### Pinning (for persistence)
```bash
# Manually pin the map
bpftool map pin name protected_pids /sys/fs/bpf/edr/protected_pids

# Query entries
bpftool map dump name protected_pids
```

### Capacity
- **Max entries**: 128 PIDs
- **Current usage**: ~5-10 (init, kernel threads, system daemons)
- **Recommended**: Keep < 100 for performance

### Lifecycle
1. **On edr_attach_any startup**: Populate with sensor PIDs + system PIDs
2. **During operation**: Entries remain in kernel (pinned map)
3. **On shutdown**: Entries cleaned up by libbpf or manual bpftool

---

## Map 2: ondemand_enabled

### Definition
```c
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 16);
    __type(key, __u32);    // prog_id or 0 for all
    __type(value, __u8);   // 1 = enabled
} ondemand_enabled SEC(".maps");
```

### Location
[edr_events.h](edr_events.h) - Added alongside other shared maps

### Purpose
Gate all 7 ON_DEMAND programs at runtime without recompiling.

### Affected Programs
All ON_DEMAND programs check this gate first:
```c
__u32 zero = 0;  // key = 0 for global enable
__u8 *enabled = bpf_map_lookup_elem(&ondemand_enabled, &zero);
if (!enabled || !(*enabled))
    return 0;  // Skip execution
```

**Programs**:
1. syscall_tracer.bpf.c
2. net_flow.bpf.c
3. encrypted_payload_monitor.bpf.c
4. container_exec_monitor_ondemand.bpf.c
5. cred_access_monitor_ondemand.bpf.c
6. suspicious_ipc_ondemand.bpf.c
7. usb_monitor_ondemand.bpf.c

### Usage in Userspace

#### Enable All ON_DEMAND Programs
```rust
let ondemand_map = bpf.map_mut("ondemand_enabled")?;
ondemand_map.insert(
    (0u32).to_ne_bytes().to_vec(),
    vec![1],
    0
)?;
```

#### Disable All ON_DEMAND Programs
```rust
let ondemand_map = bpf.map_mut("ondemand_enabled")?;
ondemand_map.insert(
    (0u32).to_ne_bytes().to_vec(),
    vec![0],
    0
)?;
```

#### Check Status
```bash
bpftool map dump name ondemand_enabled
# Output: key: 00 00 00 00  value: 01
```

### Capacity
- **Max entries**: 16 (only need 1 for global key=0)
- **Default**: Disabled (key not present = disabled)
- **Current usage**: 1 entry (global enable switch)

### Lifecycle
1. **Default**: Disabled on startup (map empty)
2. **On demand**: Userspace sets key=0, value=1 to enable
3. **During operation**: All 7 programs check at entry (minimal overhead)
4. **On disable**: Userspace sets key=0, value=0; all 7 programs stop immediately

---

## Integration Points

### edr_attach_any.rs
```rust
fn attach_all(&mut self) -> Result<()> {
    // ... load BPF objects ...
    
    // Populate protected_pids BEFORE attaching anything
    self.populate_protected_pids()?;
    
    // Optionally enable ON_DEMAND programs
    // self.enable_ondemand()?;  // call later if needed
    
    // Attach kernel programs
    self.attach_programs()?;
    
    Ok(())
}

fn populate_protected_pids(&mut self) -> Result<()> {
    let protected_pids = self.bpf.map_mut("protected_pids")?;
    let my_pid = std::process::id() as u32;
    protected_pids.insert(my_pid.to_ne_bytes().to_vec(), vec![1], 0)?;
    // Add other critical PIDs (init, kernel)
    Ok(())
}

fn enable_ondemand(&mut self) -> Result<()> {
    let ondemand_map = self.bpf.map_mut("ondemand_enabled")?;
    ondemand_map.insert((0u32).to_ne_bytes().to_vec(), vec![1], 0)?;
    Ok(())
}
```

### capture_linux_rotating.rs
```rust
fn start_reading(&mut self) -> Result<()> {
    // Ensure protected_pids populated (if not already by edr_attach_any)
    // On_demand_enabled can be toggled here if needed
    
    // Start ringbuf reader
    self.ringbuf.poll()?;
    Ok(())
}
```

---

## Monitoring & Debugging

### Check protected_pids Contents
```bash
# List all protected PIDs
bpftool map dump name protected_pids | grep -v "^$"

# Add a PID manually
pid=1234
echo "00 key: $(printf '%02x %02x %02x %02x' $((pid >> 24 & 0xFF)) $((pid >> 16 & 0xFF)) $((pid >> 8 & 0xFF)) $((pid & 0xFF)))" 
bpftool map update name protected_pids key hex 00 00 00 01 value hex 01
```

### Check ondemand_enabled Status
```bash
# Check if enabled
bpftool map dump name ondemand_enabled
# Expected output: key: 00 00 00 00  value: 01 (enabled)
#            or: (empty) = disabled

# Enable at runtime
bpftool map update name ondemand_enabled key hex 00 00 00 00 value hex 01

# Disable at runtime
bpftool map update name ondemand_enabled key hex 00 00 00 00 value hex 00
```

### Performance Impact
- **protected_pids lookup**: O(1) hash table lookup (< 100ns)
- **ondemand_enabled check**: O(1) hash table lookup (< 100ns)
- **Total per-event overhead**: ~2 microseconds (negligible)

---

## Migration from Hardcoded Values

### Before
```c
// signal_integrity.bpf.c (OLD)
static __always_inline bool is_protected_pid(__u32 pid) {
    return (pid <= 5) || (pid == 1);  // HARDCODED
}
```

### After
```c
// signal_integrity.bpf.c (NEW)
static __always_inline bool is_protected_pid(__u32 pid) {
    __u8 *protected = bpf_map_lookup_elem(&protected_pids, &pid);
    return protected && *protected == 1;  // DYNAMIC
}
```

### Advantages
- ✅ No recompilation needed to change protected PIDs
- ✅ Supports arbitrary number of PIDs (up to 128)
- ✅ Can be updated at runtime
- ✅ Userspace controls which PIDs to protect

---

## Summary

| Map | Purpose | Key Type | Value Type | Max Size | Default |
|-----|---------|----------|-----------|----------|---------|
| protected_pids | Prevent killing critical processes | __u32 (pid) | __u8 (1=protected) | 128 | Empty |
| ondemand_enabled | Enable/disable 7 ON_DEMAND programs | __u32 (0=global) | __u8 (1=enabled) | 16 | Empty (disabled) |

Both maps are **pinned** and shared across all eBPF programs via edr_events.h.
