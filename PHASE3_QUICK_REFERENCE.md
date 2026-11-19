# PHASE 3 QUICK REFERENCE

## What Changed

### 1. Runtime Path (transport.rs)
```
BEFORE: /sys/kernel/bpf/edr_events_rb
AFTER:  /sys/fs/bpf/edr/edr_events_rb
```

### 2. BPF Programs (5 files)
All now:
- `#include "edr_events.h"` (canonical)
- `_Static_assert(sizeof(struct edr_event) == 376, ...)`
- Emit to `bpf_ringbuf_reserve(&edr_events_rb, ...)`

### 3. Event Types Supported
| Category | Events |
|----------|--------|
| Process | fork, exec, exit |
| Memory | W→X, memfd/exec, RWX mmap |
| Kernel | ptrace, module_load, delete_module |
| Network | TCP states, retransmissions |

---

## Verification Commands

```bash
# Check BPF programs have assertions
grep -l "_Static_assert.*376" linux/ebpf/*.bpf.c

# Check transport path is correct
grep "/sys/fs/bpf/edr" linux/ebpf/transport.rs

# Build and test
cargo build --all-targets
cargo test -q

# After programs loaded, run self-test
./target/release/forensic_hooks capture --self-test-ebpf
```

---

## Files Modified

| File | Changes | Status |
|------|---------|--------|
| proc_lifecycle.bpf.c | +376B struct init | ✅ |
| wx_exec.bpf.c | +edr_event emitter | ✅ |
| raw_syscalls_catcher.bpf.c | +assertion | ✅ |
| tcp_state.bpf.c | +assertion | ✅ |
| mem_scan.bpf.c | +edr_event init | ✅ |
| transport.rs | +path fix | ✅ |

---

## Next: Phase 4

```bash
# 1. Compile BPF
clang -O2 -target bpf -c linux/ebpf/*.bpf.c

# 2. Load and pin
mkdir -p /sys/fs/bpf/edr
# Load programs (aya/libbpf)

# 3. Test
./target/release/forensic_hooks capture --self-test-ebpf
# Expected: events_read > 0
```

---

## Success Criteria

✅ All BPF programs use canonical struct (376 bytes)  
✅ All events emit to edr_events_rb  
✅ Transport checks correct path (/sys/fs/bpf/edr)  
✅ Build passes (0 errors, 0 warnings)  
✅ Tests pass (7/7)  
⏳ Awaiting: BPF compilation and kernel loading
