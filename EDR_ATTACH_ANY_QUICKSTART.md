# edr_attach_any - Quick Start Guide

## Installation

```bash
# Build the binary
cargo build --release --bin edr_attach_any

# Build all eBPF programs (happens automatically with cargo build)
cargo build
```

The binary is now at: `./target/release/edr_attach_any`

---

## Basic Usage

### 1. Attach Default (ALWAYS_ON) Programs
```bash
sudo ./edr_attach_any --bpf-dir build/bpf --verbose
```

This attaches all 10 ALWAYS_ON programs:
- proc_lifecycle, wx_exec, tcp_connect, privilege_transitions, file_ops_guard
- exec_provenance, ptrace_injection, kernel_module_load, signal_integrity, dns_telemetry

**Expected Output:**
```
[edr_attach_any] BPF loader starting
[edr_attach_any] bpf_dir: "build/bpf"
[edr_attach_any] pin_root: "/sys/fs/bpf/edr"
[edr_attach_any] ✓ RLIMIT_MEMLOCK raised to 512 MB
[edr_attach_any] ✓ Pin directory: "/sys/fs/bpf/edr"
[edr_attach_any]   + proc_lifecycle.bpf.o
[edr_attach_any]   + wx_exec.bpf.o
... (9 more programs)
[edr_attach_any] ✓ ALWAYS_ON programs attached (10)
[edr_attach_any] ✓ Ready
```

---

### 2. Investigation Mode (All ON_DEMAND for 30 seconds)
```bash
sudo ./edr_attach_any --bpf-dir build/bpf --enable on_demand --ttl-seconds 30 --verbose
```

This attaches:
- All 10 ALWAYS_ON programs
- All 7 ON_DEMAND programs (syscall_tracer, net_flow, encrypted_payload, etc.)
- Exits automatically after 30 seconds

Use this for:
- Threat hunting sessions
- Incident investigation windows
- Performance monitoring
- Debugging specific behaviors

---

### 3. Attach Specific Programs Only
```bash
# Single program
sudo ./edr_attach_any --enable privilege_transitions

# Multiple specific programs
sudo ./edr_attach_any --enable privilege_transitions --enable kernel_module_load

# ALWAYS_ON + specific ON_DEMAND
sudo ./edr_attach_any --enable syscall_tracer_ondemand --enable net_flow_ondemand --ttl-seconds 60
```

---

### 4. Custom BPF Directory
```bash
# If you've pre-compiled to a different location
sudo ./edr_attach_any --bpf-dir /opt/forensics/bpf --verbose
```

---

### 5. Custom Pin Root (for debugging)
```bash
# Use a custom BPFfs mount for testing
sudo ./edr_attach_any --bpf-dir build/bpf --pin-root /tmp/bpf_test
```

---

## Command-Line Reference

```
edr_attach_any - BPF program loader and attacher

Usage: edr_attach_any [OPTIONS]

OPTIONS:
  --bpf-dir DIR              BPF object directory (default: build/bpf)
  --pin-root PATH            Map pin root (default: /sys/fs/bpf/edr)
  --enable <name>            Attach ON_DEMAND program by name
  --enable on_demand         Attach ALL ON_DEMAND programs
  --ttl-seconds N            Exit after N seconds
  -v, --verbose              Verbose output
  --help                     Show this message and list programs

ALWAYS_ON programs (attached by default):
  - proc_lifecycle.bpf.o
  - wx_exec.bpf.o
  - tcp_connect.bpf.o
  - privilege_transitions.bpf.o
  - file_ops_guard.bpf.o
  - exec_provenance.bpf.o
  - ptrace_injection.bpf.o
  - kernel_module_load.bpf.o
  - signal_integrity.bpf.o
  - dns_telemetry.bpf.o

ON_DEMAND programs (use --enable):
  - syscall_tracer_ondemand.bpf.o
  - net_flow_ondemand.bpf.o
  - encrypted_payload_monitor.bpf.o
  - container_exec_monitor_ondemand.bpf.o
  - suspicious_ipc.c
  - cred_access_monitor_ondemand.bpf.o
  - usb_monitor_ondemand.bpf.o
```

---

## Event Consumption

### With capture_linux_rotating

While edr_attach_any is running, read events in another terminal:

```bash
sudo ./target/debug/capture_linux_rotating --ringbuf /sys/fs/bpf/edr/edr_events_rb --verbose
```

This reads events from the shared ringbuf and prints them in real-time.

---

## Troubleshooting

### "edr_attach_any: Permission denied"
```bash
# Must run with sudo
sudo ./edr_attach_any --bpf-dir build/bpf
```

### "RLIMIT_MEMLOCK failed"
```bash
# May need higher limits if using heavy ON_DEMAND programs
ulimit -l unlimited
sudo ./edr_attach_any --bpf-dir build/bpf --enable on_demand
```

### "BPF object not found"
```bash
# Ensure build/bpf directory exists and contains compiled .o files
ls -la build/bpf/*.bpf.o
cargo build  # Rebuilds eBPF objects
```

### "Pin directory permission denied"
```bash
# Default /sys/fs/bpf/edr may need sudo or custom mount
sudo ./edr_attach_any --pin-root /sys/fs/bpf/edr --verbose
```

---

## Performance Expectations

### ALWAYS_ON Programs (All 10 Attached)
- **Event Rate**: ~100 events/sec total
- **CPU Overhead**: ~2-5% single-core equivalent
- **Memory**: ~50-100 MB ringbuf + maps
- **Stability**: Safe for 24/7 operation
- **Reserve Failed Rate**: <0.1%

### + ON_DEMAND Programs (All 7 Additional)
- **Event Rate**: ~1000-2000 events/sec (depends on sampling)
- **CPU Overhead**: ~10-30% single-core equivalent
- **Memory**: ~200-300 MB total
- **Stability**: Safe for 1-60 minute investigation windows
- **Reserve Failed Rate**: <1% (may spike during high activity)

### Sampling Impact
- **syscall_tracer_ondemand** with SAMPLE_RATE=100: ~10 events/sec
- **net_flow_ondemand** with per-flow sampling: ~50 events/sec
- **encrypted_payload_monitor** with entropy filtering: ~1-5 events/sec

---

## Integration with SIEM/Detection Systems

### Event Format
All programs emit struct edr_event (384 bytes):
```c
struct edr_event {
    uint32_t abi_version;      // 1
    uint32_t event_size;       // 384
    uint32_t event_type;       // Type-specific
    uint32_t pid;
    uint32_t uid;
    uint32_t timestamp_ns;
    // ... event-specific data
};
```

### Reading from Ringbuf
Use the Rust decoder in [linux/ebpf/reader.rs](linux/ebpf/reader.rs):
```rust
let rb = RingBuffer::new();
for event in rb.poll_events() {
    println!("Event: {:?}", event);
}
```

---

## Next Steps

1. **Run in Investigation Mode**
   ```bash
   sudo ./edr_attach_any --bpf-dir build/bpf --enable on_demand --ttl-seconds 30
   ```

2. **Read Events in Another Terminal**
   ```bash
   sudo ./target/debug/capture_linux_rotating --verbose
   ```

3. **Analyze Event Patterns**
   - Look for suspicious process chains
   - Monitor file access patterns
   - Detect network anomalies

4. **Tune Sampling for Your Environment**
   - Edit ON_DEMAND program sampling constants
   - Recompile: `cargo build`
   - Re-run with new sampling rates

5. **Deploy to Production**
   - Copy edr_attach_any + BPF .o files to target system
   - Create systemd service for startup
   - Configure event forwarding to SIEM

---

## References

- **Full Program Inventory**: [EBPF_PROGRAM_INVENTORY.md](EBPF_PROGRAM_INVENTORY.md)
- **Architecture Guide**: [SHARED_BASELINE_ARCHITECTURE.md](SHARED_BASELINE_ARCHITECTURE.md)
- **Integration Guide**: [BASELINE_INTEGRATION_GUIDE.md](BASELINE_INTEGRATION_GUIDE.md)
