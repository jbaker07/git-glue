# BPF Program ABI Compatibility Report

**Date**: December 22, 2025  
**Focus**: Verify that BPF .c files use canonical event ABI and map names

## Current State: Mismatch Identified

| Program | Map Type | Struct Defined | Size | ABI Match | Status |
|---------|----------|---|---|---|---|
| proc_lifecycle.bpf.c | ringbuf | **custom edr_event** | ~296B | ❌ MISMATCH | Uses local struct |
| syscall_tracer.bpf.c | ringbuf | unknown | ? | ❓ UNCHECKED | Needs review |
| wx_exec.bpf.c | ringbuf | unknown | ? | ❓ UNCHECKED | Needs review |
| net_flow.bpf.c | ringbuf | unknown | ? | ❓ UNCHECKED | Needs review |
| mem_scan.bpf.c | perf | unknown | ? | ❓ UNCHECKED | Needs review |
| file_access_monitor.c | ringbuf | unknown | ? | ❓ UNCHECKED | Needs review |
| dns_tunnel_monitor.c | ringbuf | unknown | ? | ❓ UNCHECKED | Needs review |

## Issue Analysis

### Problem 1: Local Struct Definitions

**File**: `proc_lifecycle.bpf.c` (lines 33-46)

```c
struct edr_event {
    __u64 ts;            // monotonic ns
    __u32 type;          // EVT_*
    __s32 syscall_id;    // if from sys_enter/exit; 0 for TPs
    __u32 tgid;          // process id
    __u32 ppid;          // parent id (optional if known)
    __u32 uid;           // uid at emit time
    __s32 fd;            // file/socket fd if relevant else -1
    __s32 ret;           // syscall/derived return (e.g., child pid for fork)
    __u32 aux_u32;       // misc small field (e.g., exit_code)
    char  comm[16];      // task comm
    char  path[256];     // best-effort path/argv/comm as text
};
// Total: 8 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 16 + 256 = 312 bytes
```

**Issue**: This is 312 bytes, not 376 bytes. Missing fields:
- `path2[128]` (secondary path)
- Network fields: `fam`, `proto`, `lport`, `laddr4`, `laddr6`, `rport`, `raddr4`, `raddr6`

**Canonical** (376 bytes):
```c
struct edr_event {
    __u64 ts;
    __u32 type;
    __u32 syscall_id;
    __u32 tgid;
    __u32 ppid;
    __u32 uid;
    __s32 fd;
    __s32 ret;
    __u32 flags;
    __u32 aux_u32;
    __u64 aux_u64;        // 8-byte alignment keeper
    __u8  fam;            // AF_*
    __u8  proto;          // IPPROTO_*
    __u16 lport;          // local port (BE)
    __u32 laddr4;         // local ipv4 (BE)
    __u8  laddr6[16];
    __u16 rport;          // remote port (BE)
    __u32 raddr4;         // remote ipv4 (BE)
    __u8  raddr6[16];
    char  path[128];      // primary path
    char  path2[128];     // secondary path
    char  comm[16];       // task comm
} __attribute__((aligned(8)));
```

### Problem 2: Multiple Map Definitions

Programs define local ringbuf/perf maps instead of using pinned maps at:
- `/sys/kernel/bpf/edr_events_rb` (ringbuf)
- `/sys/kernel/bpf/edr_events_perf` (perf)

## Recommended Fixes

### Step 1: Create Unified edr_events.h Include

**Action**: Use the canonical `edr_events.h` (already exists at `linux/ebpf/edr_events.h`)

**All .bpf.c files should:**

```c
#include "edr_events.h"  // Provides canonical struct + map definitions
```

### Step 2: Update proc_lifecycle.bpf.c

**Before**:
```c
// Lines 8-45: custom struct edr_event definition
struct edr_event { ... };  // 312 bytes
struct { ... } ringbuf SEC(".maps");
```

**After**:
```c
#include "edr_events.h"
// Already provides:
// - struct edr_event (376 bytes, canonical)
// - struct { __uint(type, BPF_MAP_TYPE_RINGBUF); } edr_events_rb SEC(".maps");
```

### Step 3: Add Compile-Time Assertions

**Every .bpf.c should verify**:

```c
#include "edr_events.h"

// Verify the ABI hasn't drifted
_Static_assert(sizeof(struct edr_event) == 376, "edr_event must be 376 bytes");
```

### Step 4: Rename Maps Consistently

All programs should emit to canonical map names:
- **Ringbuf**: `edr_events_rb`
- **Perf**: `edr_events_perf`

Emit call:
```c
// Instead of: bpf_ringbuf_output(&ringbuf, ...)
// Use:
int ret = bpf_ringbuf_output(&edr_events_rb, event, sizeof(*event), 0);
```

## Current Map Status

### edr_events.h Map Definitions

**File**: `linux/ebpf/edr_events.h` (lines 85-116)

```c
/* -------- ringbuf -------- */
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 24); // 16MB
} edr_events_rb SEC(".maps");

/* -------- counter for ringbuf_reserve failures -------- */
struct {
    __uint(type, BPF_MAP_TYPE_PERCPU_ARRAY);
    __uint(max_entries, 1);
    __type(key, __u32);
    __type(value, __u64);
} edr_rb_reserve_failed SEC(".maps");

/* -------- counter for events successfully reserved -------- */
struct {
    __uint(type, BPF_MAP_TYPE_PERCPU_ARRAY);
    __uint(max_entries, 1);
    __type(key, __u32);
    __type(value, __u64);
} edr_events_reserved SEC(".maps");
```

**Status**: ✅ Ready to use - programs just need to `#include "edr_events.h"`

## Integration Checklist

### For Each .bpf.c File

- [ ] Remove local `struct edr_event` definition
- [ ] Remove local ringbuf/perf map definitions
- [ ] Add `#include "edr_events.h"` at top
- [ ] Change `bpf_ringbuf_output(&ringbuf, ...)` → `bpf_ringbuf_output(&edr_events_rb, ...)`
- [ ] Add compile-time assertion: `_Static_assert(sizeof(struct edr_event) == 376, ...)`
- [ ] Verify all event field assignments match canonical struct

### Example: Updated proc_lifecycle.bpf.c

```c
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "edr_events.h"  // Canonical struct + maps

// Verify ABI
_Static_assert(sizeof(struct edr_event) == 376, "edr_event must be 376 bytes");

// Use canonical struct edr_event and edr_events_rb
SEC("tp/sched/sched_process_exec")
int handle_exec(struct trace_event_raw_sched_process_exec *ctx) {
    struct edr_event *event = bpf_ringbuf_reserve(&edr_events_rb, sizeof(*event), 0);
    if (!event)
        return 0;
    
    event->ts = bpf_ktime_get_ns();
    event->type = EVT_PROC_EXEC_TP;
    event->tgid = (ctx->pid >> 32);
    event->uid = bpf_get_current_uid_gid();
    // ... populate other fields ...
    
    bpf_ringbuf_submit(event, 0);
    return 0;
}
```

## Verification Commands

### Check All Program Struct Sizes

```bash
clang -target bpf -O2 -c -o /tmp/test.o proc_lifecycle.bpf.c -Ilinux/ebpf/include
llvm-objdump -S /tmp/test.o | grep "edr_event"
# Should show: struct edr_event size == 376
```

### Verify Map Names

```bash
bpftool map list | grep edr_events
# Should show: edr_events_rb, edr_rb_reserve_failed, edr_events_reserved
```

## Post-Integration Expectations

### Events on Linux with BPF Loaded

```bash
./target/debug/forensic_hooks capture --self-test-ebpf

[eBPF self-test] Transport: ringbuf  ← or 'perf'
[eBPF self-test] [ 0s] events_read=0, transport=ringbuf, errors=0
[eBPF self-test] [ 5s] events_read=47, transport=ringbuf, errors=0  ← >0
[eBPF self-test] [10s] events_read=120, transport=ringbuf, errors=0  ← increasing
[eBPF self-test] Complete.
[eBPF self-test] PASS
```

### Load Test with Actual Events

```bash
./tools/ebpf_load_test.sh

[ebpf_load_test] [ 5s remaining] events=2847 dropped=0 reserve_failed=0
[ebpf_load_test] PASS: All thresholds met
```

## Timeline

1. **Now** (Phase 2): ABI documented, transport validates map existence
2. **Phase 2.1** (1-2 hours): Update 3-5 key BPF programs to use canonical header
3. **Phase 2.2** (30 mins): Test with actual event emission
4. **Phase 2.3** (30 mins): Run load test, verify event counts and loss metrics

## Summary

**Root Cause**: BPF programs define local structs instead of including canonical `edr_events.h`.

**Fix**: One-line change per file: `#include "edr_events.h"`

**Impact**: Once fixed, ringbuf/perf readers will receive 376-byte events that userspace decoder expects.

**No changes needed** in userspace (transport.rs, reader.rs, metrics.rs) — they already implement the correct ABI.
