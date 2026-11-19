## Phase 4: Real Kernel Integration - ABI Safety Status

### ✅ COMPLETE: ABI Versioning Framework

**Struct Changes**:
- struct edr_event: now 384 bytes (was 376)
- First 8 bytes: abi_version (u32=1) + event_size (u32=384)
- All 5 BPF programs enforce via _Static_assert(sizeof(struct edr_event) == 384)

**Files Modified**:
1. linux/ebpf/edr_events.h - struct definition + edr_emit() helper
2. linux/ebpf/transport.rs - RawEbpfEvent + validate_abi_event()
3. linux/ebpf/proc_lifecycle.bpf.c - fill_common() sets ABI fields
4. linux/ebpf/wx_exec.bpf.c - emit_wx_evt() sets ABI fields
5. linux/ebpf/raw_syscalls_catcher.bpf.c - ev_reserve_zero() sets ABI fields
6. linux/ebpf/tcp_state.bpf.c - uses edr_emit() helper
7. linux/ebpf/mem_scan.bpf.c - trace_mmap() + trace_execve_mem() set ABI fields
8. src/bin/edr_attach_any.rs - cleaned unused import

**Build Status**: ✓ Clean (0 warnings, 0 errors)

**Next Steps**:
- [ ] Implement actual BPF program loading in edr_attach_any
- [ ] Create kernel integration test
- [ ] Verify ringbuf map pinning
- [ ] Run capture --self-test-ebpf with ABI validation
