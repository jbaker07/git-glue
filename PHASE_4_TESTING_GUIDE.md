# Phase 4: Multi-Platform Testing & Integration

**Status**: Ready for Implementation  
**Created**: 2024-01-31  
**Purpose**: Validate core EDR functionality across Linux, macOS, and Windows

---

## Executive Summary

Phase 4 validates that the EDR platform can:
- **Collect events** from heterogeneous platforms (Linux, macOS, Windows)
- **Unify event formats** across platforms using a standardized schema
- **Attach eBPF programs** (Linux) and native sensors (macOS/Windows)
- **Load and execute playbooks** across platforms
- **Verify event delivery** through ring buffers and IPC
- **Score signals** for security relevance

---

## Pre-Flight Checklist

Before starting Phase 4, run the verification script:

```bash
./scripts/verify_phase4_readiness.sh
```

This checks:
- ✓ All required source files exist
- ✓ Platform modules (Linux, macOS, Windows) are present
- ✓ Playbook structure is correct
- ✓ Dependencies are declared (Aya, libbpf, etc.)
- ✓ Code compiles on current platform

Expected output:
```
✓ System is ready for Phase 4 testing!

Next steps:
  1. Compile: cargo build --release
  2. Run tests: cargo test
  3. Start capture: cargo run --bin forensic_hooks --release
```

---

## Compilation Strategy

### Build for Current Platform

```bash
# Compile the main binary and all binaries
cargo build --release

# Binaries created:
# - target/release/forensic_hooks       (main EDR capture)
# - target/release/entmon_attach        (entropy monitor; hooks signal engine)
# - target/release/fingerprint_gen      (fingerprint generator)
# - target/release/edr_attach_any       (BPF loader for Linux)
```

### Platform-Specific Builds

**Linux** (with eBPF support):
```bash
cargo build --release --features "ebpf,forensic"
```

**macOS** (native sensors):
```bash
cargo build --release --features "forensic"
# macOS does not use eBPF; uses native APIs instead
```

**Windows** (ETW collection):
```bash
cargo build --release --features "windows_etw,forensic"
# Note: Windows compilation only works on Windows
```

---

## Testing Approach

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test '*'
```

### Self-Test Mode (No Root Required)
```bash
./target/release/forensic_hooks --self-test-ebpf

# Expected output:
# [forensic_hooks] Starting EDR capture (self-test mode)
# [forensic_hooks] Playbook loading complete: 45 playbooks
# [forensic_hooks] Signal orchestrator ready
# [forensic_hooks] Event schema validated: abi_version=1, event_size=384
```

---

## Phase 4 Test Scenarios

### Scenario 1: Event Schema Validation
**Objective**: Verify unified event format across platforms

```bash
# Start capture in event schema dump mode
./target/release/forensic_hooks --dump-event-schema

# Expected: JSON schema showing unified event format with:
# - timestamp, uuid, event_type, source (linux/macos/windows)
# - process_context (ppid, uid, gid, comm)
# - syscall_args (up to 6 args)
# - result, errno
```

### Scenario 2: Playbook Loading
**Objective**: Verify playbooks load correctly across platforms

```bash
./target/release/forensic_hooks --validate-playbooks

# Expected: List of all playbooks by category:
# Common: 15 playbooks
# Linux-specific: 12 playbooks
# macOS-specific: 10 playbooks
# Windows-specific: 8 playbooks
```

### Scenario 3: Signal Orchestration
**Objective**: Verify signal engine routes events to correct rules

```bash
./target/release/forensic_hooks --test-signal-engine

# Expected: Statistics on rule matches and signal generation:
# Total rules: 45
# Signal types: privilege_escalation, persistence, defense_evasion, ...
```

### Scenario 4: eBPF Attachment (Linux Only)
**Objective**: Load and attach BPF programs

```bash
# Terminal 1: Start the attach loader
sudo ./target/release/edr_attach_any

# Terminal 2: Verify BPF programs are loaded
sudo bpftool prog list
sudo bpftool map list

# Expected: eBPF programs pinned under /sys/fs/bpf/edr/
```

### Scenario 5: Event Ring Buffer
**Objective**: Verify ring buffer delivers events

```bash
# Linux only - requires elevated privileges
sudo ./target/release/forensic_hooks --ring-buffer-test

# Expected: Events flowing through ring buffer with:
# - Event count
# - Average latency
# - Ring buffer utilization
```

---

## Architecture Components

### Binaries

| Binary | Purpose | Platform | Requires Root |
|--------|---------|----------|---------------|
| `forensic_hooks` | Main EDR capture engine | All | Linux/macOS/Windows |
| `entmon_attach` | Entropy monitor & signal engine | All | No |
| `fingerprint_gen` | Playbook fingerprint generator | All | No |
| `edr_attach_any` | eBPF program loader | Linux | Yes |

### Module Structure

```
src/
├── lib.rs                 # Platform-agnostic core
├── linux/                 # Linux-specific (eBPF, kprobes, tracepoints)
├── macos/                 # macOS-specific (native APIs, Kauth, ESF)
├── windows/               # Windows-specific (ETW, WMI)
└── locald/                # Local daemon (signal engine, playbook loader)

playbooks/
├── common/                # Rules for all platforms
├── linux/                 # Linux-specific rules
├── macos/                 # macOS-specific rules
└── windows/               # Windows-specific rules
```

### Event Flow

```
Platform Sensors
    ↓
Event Schema Unification
    ↓
Ring Buffer (eBPF) / IPC (macOS/Windows)
    ↓
Signal Orchestrator
    ↓
Playbook Matching
    ↓
Rule Scoring & Alert Generation
```

---

## Troubleshooting

### Compilation Errors

**Error**: `error: cannot find crate 'aya'`
- **Solution**: Run on Linux or use `--no-default-features`

**Error**: `error: cannot find crate 'openssl'`
- **Solution**: Install OpenSSL development headers
  - macOS: `brew install openssl`
  - Linux: `sudo apt-get install libssl-dev`
  - Windows: `vcpkg install openssl:x64-windows`

### Runtime Errors

**Error**: `[edr_attach_any] getrlimit(RLIMIT_MEMLOCK) failed`
- **Solution**: Run with root (`sudo`) and ensure adequate memory

**Error**: `[forensic_hooks] Ring buffer not available`
- **Solution**: On Linux, ensure kernel ≥ 5.8 and BPF ring buffer support

### Platform-Specific Issues

**macOS**: Kauth requires extension approval
- Use `spctl --assess --type execute <binary>` to verify code signing

**Windows**: ETW requires Event Tracing Service
- Ensure EVENTSYSTEM service is running: `sc query EVENTSYSTEM`

---

## Validation Checklist

- [ ] `verify_phase4_readiness.sh` passes all checks
- [ ] `cargo build --release` completes without errors
- [ ] `forensic_hooks --self-test-ebpf` validates event schema
- [ ] Playbooks validate with `--validate-playbooks`
- [ ] Signal engine responds to events with `--test-signal-engine`
- [ ] Platform-specific sensors attach correctly
- [ ] Ring buffer (Linux) / IPC (macOS/Windows) delivers events
- [ ] Signal scoring produces expected outputs

---

## Next Steps

Once Phase 4 validation is complete:

1. **Phase 5**: Advanced scoring (YARA, Elastic, ECS)
2. **Phase 6**: Distributed deployment (fleet management)
3. **Phase 7**: Incident response automation (playbook execution)

---

## Reference Documentation

- [BASELINE_SYSTEM_DESIGN.md](../BASELINE_SYSTEM_DESIGN.md) - Architecture overview
- [BASELINE_IMPLEMENTATION_SUMMARY.md](../BASELINE_IMPLEMENTATION_SUMMARY.md) - Implementation status
- [BASELINE_INTEGRATION_GUIDE.md](../BASELINE_INTEGRATION_GUIDE.md) - Integration steps
- [WINDOWS_SIGNAL_ENGINE_INTEGRATION.md](../WINDOWS_SIGNAL_ENGINE_INTEGRATION.md) - Windows details

---

**Created by**: Forensic Hooks Phase 4 Implementation  
**Last Updated**: 2024-01-31
