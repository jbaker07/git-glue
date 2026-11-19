# Attic: macos_linux_copies/

This directory contains Linux-specific code that was incorrectly placed in `macos/sensors/`.

## Files in this directory

### Linux eBPF code (should be in linux/ if needed)
- **mem_scan.rs** - eBPF-based memory scanning using `aya` crate
  - Uses: aya, Ebpf, TracePoint, perf events
  - Reason for move: This is a Linux kernel monitoring primitive, not macOS-native

- **container_monitor.rs** - eBPF-based container monitoring using `aya` crate
  - Uses: aya, Ebpf, RingBuf, KProbe, RawTracePoint
  - Reason for move: Linux container monitoring (cgroups, namespaces), not applicable to macOS

- **privilege_monitor.rs** - eBPF-based privilege escalation monitoring
  - Uses: aya, Ebpf, TracePoint, perf events
  - Reason for move: Linux kernel capability monitoring

- **usb_monitor.rs** - eBPF-based USB device monitoring using `aya` crate
  - Uses: aya, Bpf, TracePoint, lsblk command
  - Reason for move: Linux-specific device enumeration and eBPF tracing

## Proper Locations

If Linux sensor support is needed in the future:
- Create `linux/sensors/` directory
- Move these files there (with adaptations for Linux event sources)
- Keep macOS sensors in `macos/sensors/` (BSM + ES extractors only)

## Why This Move?

The macOS `/sensors/` directory should contain ONLY:
- **bsm/**: OpenBSM audit log extraction
- **es/**: Endpoint Security framework extraction
- **Detectors**: Cross-platform signal detection (e.g., geo_ip_anomaly, password_spray)

NOT:
- eBPF kernel probes (Linux only)
- Linux device enumeration (lsblk, /proc)
- Linux capability monitoring
- Linux container monitoring

This cleanup ensures:
1. **Clarity**: macos/sensors = macOS capture + extraction
2. **Separation**: Linux code lives in linux/ when ready
3. **Compilability**: Removing aya/Ebpf dependencies from macOS build

## Recovery

If you need this code later:
```bash
git mv attic/macos_linux_copies/mem_scan.rs linux/sensors/mem_scan.rs
# (and similar for others)
```

All files are preserved as-is for reference.
