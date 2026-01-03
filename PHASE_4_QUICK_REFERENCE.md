# Phase 4 Quick Reference

**Quick Start for Phase 4 Testing**

## Prerequisites Check (< 1 minute)
```bash
cd /Users/jermainebaker/git-glue
./scripts/verify_phase4_readiness.sh
```
Expected: All 23 checks pass ✓

## Build (2-5 minutes)
```bash
# Current platform
cargo build --release

# Specific platform (must run on that OS)
cargo build --release --features "ebpf,forensic"  # Linux with eBPF
cargo build --release --features "forensic"       # macOS
cargo build --release --features "windows_etw,forensic"  # Windows
```

## Quick Tests (No Root Required)
```bash
# Self-test
./target/release/forensic_hooks --self-test-ebpf

# Validate playbooks
./target/release/forensic_hooks --validate-playbooks

# Test signal engine
./target/release/forensic_hooks --test-signal-engine

# Dump event schema
./target/release/forensic_hooks --dump-event-schema
```

## Root-Required Tests (Linux Only)
```bash
# Attach eBPF programs
sudo ./target/release/edr_attach_any

# Ring buffer test
sudo ./target/release/forensic_hooks --ring-buffer-test

# Verify BPF programs
sudo bpftool prog list
sudo bpftool map list
```

## Troubleshooting

**Build fails**: Run on target OS, install dependencies (openssl-dev on Linux)
**edr_attach_any fails**: Need root (`sudo`), kernel ≥ 5.8 on Linux
**Compilation errors**: See PHASE_4_TESTING_GUIDE.md

## Binaries Created

| Binary | Location | Purpose |
|--------|----------|---------|
| forensic_hooks | target/release/forensic_hooks | Main EDR capture |
| entmon_attach | target/release/entmon_attach | Signal orchestrator |
| fingerprint_gen | target/release/fingerprint_gen | Rule fingerprinting |
| edr_attach_any | target/release/edr_attach_any | eBPF loader (Linux) |

## Key Files

| File | Purpose |
|------|---------|
| PHASE_4_COMPLETION_REPORT.md | Full verification results |
| PHASE_4_TESTING_GUIDE.md | Detailed testing procedures |
| PHASE_4_IMPLEMENTATION_SUMMARY.md | What was built |
| scripts/verify_phase4_readiness.sh | Automated checks |

## Event Schema

```
384-byte unified event:
- timestamp (u64)
- uuid (16 bytes)
- event_type (4 bytes)
- source (4 bytes: linux/macos/windows)
- process_context (64 bytes)
- syscall_args (6 × u64)
- result (i64)
- errno (i32)
```

## Playbooks

45 total rules:
- Common: 15 rules (all platforms)
- Linux: 12 rules
- macOS: 10 rules
- Windows: 8 rules

Location: `playbooks/{common,linux,macos,windows}/`

## Documentation

Start here: **PHASE_4_COMPLETION_REPORT.md**

Then: **PHASE_4_TESTING_GUIDE.md**

Reference: **BASELINE_*.md** files
