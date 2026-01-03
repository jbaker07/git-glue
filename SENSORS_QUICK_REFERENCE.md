# Linux Sensors v2 - Quick Reference

## All 22 Sensors at a Glance

| # | Sensor | OS Source | Check 1 (High-Precision) | Check 2 (Heuristic) | Dedup Strategy |
|---|--------|-----------|--------------------------|-------------------|-----------------|
| 1 | **alert_engine** | In-memory events | Event burst ≥10 in 5s | Buffer overflow >1000 | rate_limit 5s |
| 2 | **auth_pipe_listener** | /var/run/edr-auth.fifo | JSONL format valid | Failed auth pattern | rate_limit 5s per user |
| 3 | **cloud_tracker** | /proc, ~/.aws, ~/.config | CLI detection (5 tools) | Config changes | seen_once per CLI, rate_limit 1h |
| 4 | **container_monitor** | /var/run/*.sock, cgroup | Docker socket exists | Cgroup container | seen_once socket, rate_limit 1m PID |
| 5 | **dll_injection_monitor** | /proc/*/environ, /maps | LD_PRELOAD present | RWX anon memory ≥2 | seen_once env, rate_limit 1m |
| 6 | **encrypted_payload_detector** | Entropy scan /tmp, crypto execs | Entropy >7.0 | Crypto tool exec | rate_limit 1h |
| 7 | **entropy_exec_monitor** | Writable path executables | Exec bit in /tmp | Quick entropy heuristic | seen_once exec, rate_limit 1m |
| 8 | **file_hash_watcher** | Monitored binaries (5 critical) | Hash changed | IOC list match | changed_hash, rate_limit 1h |
| 9 | **geo_ip_anomaly** | /proc/net/tcp, /proc/*/fd | New remote IP | Socket count anomaly | seen_once IP, rate_limit 1m |
| 10 | **job_sched_monitor** | /etc/cron.*, systemd units | Suspicious tokens | Systemd anomaly | rate_limit 1h per file |
| 11 | **lkm_monitor** | /proc/modules, /sys/module | New module loaded | License missing | seen_once module, rate_limit 1h |
| 12 | **logon_tracker** | who, lastb, lastlog | Session tracking | System user login | seen_once session, rate_limit 1h |
| 13 | **mem_scan** | /proc/*/maps | RWX region found | W→X transition | seen_once region, rate_limit 1m |
| 14 | **mfa_bypass** | /etc/pam.d/*, sshd_config | MFA disabled in PAM | Weak SSH setting | rate_limit 1h per config |
| 15 | **password_spray** | /var/log/auth.log, lastb | Failed ≥5 per IP in 5m | lastb pattern analysis | rate_limit 5m per IP |
| 16 | **replay_writer** | None (demo only) | EDR_DEMO=1 check | Synthetic events | rate_limit 1h demo init |
| 17 | **script_monitor** | /tmp, /var/tmp, /dev/shm | Script with shebang | Interp from writable | seen_once script, rate_limit 1h |
| 18 | **signal_integrity_mapper** | /proc/modules vs /sys/module | Orphaned module | Network inconsistency | seen_once orphaned, rate_limit 1h |
| 19 | **suspicious_ipc** | /proc/net/unix, /dev/shm | World-writable socket | Suspicious shm name | seen_once socket, rate_limit 1h |
| 20 | **telemetry_fingerprint** | /etc/edr/fingerprints.json | Database load | N/A (read-only) | N/A - no events |
| 21 | **trust_state_restorer** | /sys/firmware, /sys/kernel | Trust state summary | N/A (read-only state) | rate_limit 1h summary |
| 22 | **usb_monitor** | /sys/bus/usb/devices | Device insertion | Risk by class (HID, storage) | seen_once device, rate_limit 1h |

---

## Event Types Emitted

### Alert & Aggregation
- `alert_engine`: `alert_candidate`

### Authentication
- `auth_pipe_listener`: `auth_event`
- `logon_tracker`: `login_session`, `system_user_login`, `remote_login`

### Cloud & Containers
- `cloud_tracker`: `cloud_execution`, `cloud_config`
- `container_monitor`: `container_socket`, `container_process`

### Code Injection & Memory
- `dll_injection_monitor`: `ld_injection`, `suspicious_mmap`
- `mem_scan`: `rwx_memory`, `multiple_rwx`, `wx_transition`

### Data & Encryption
- `encrypted_payload_detector`: `high_entropy`, `crypto_tool_exec`
- `entropy_exec_monitor`: `exec_from_writable`, `suspicious_exec`
- `file_hash_watcher`: `file_modified`, `ioc_detected`

### Network & IPC
- `geo_ip_anomaly`: `new_remote_ip`, `connection_anomaly`
- `password_spray`: `password_spray`
- `suspicious_ipc`: `suspicious_socket`, `world_writable_socket`, `suspicious_shm`, `large_shm`

### System & Kernel
- `job_sched_monitor`: `suspicious_cron`, `suspicious_systemd`
- `lkm_monitor`: `kernel_module_loaded`, `module_tamper`
- `signal_integrity_mapper`: `integrity_violation`

### Scripts & Execution
- `script_monitor`: `script_created`, `home_script`, `interpreter_exec_writable`

### Security & Trust
- `mfa_bypass`: `mfa_bypass_suspect`
- `trust_state_restorer`: `sensor_health`
- `usb_monitor`: `usb_device_inserted`, `risky_usb_device`

### Demo
- `replay_writer`: `demo_event` (only if EDR_DEMO=1)

---

## Rate-Limit Strategies Used

| Strategy | Sensors | Purpose |
|----------|---------|---------|
| **seen_once** | alert_engine, auth_pipe_listener, cloud_tracker, container_monitor, dll_injection_monitor, encrypted_payload_detector, entropy_exec_monitor, file_hash_watcher, geo_ip_anomaly, lkm_monitor, mem_scan, script_monitor, suspicious_ipc, signal_integrity_mapper | Track first occurrence of unique item |
| **changed_hash** | file_hash_watcher | Detect changes in monitored files |
| **rate_limit** (5s) | alert_engine, auth_pipe_listener, encrypted_payload_detector, mem_scan, script_monitor | Quick throttle (spam prevention) |
| **rate_limit** (1m) | container_monitor, dll_injection_monitor, geo_ip_anomaly, mem_scan, script_monitor | Per-minute throttle |
| **rate_limit** (5m) | password_spray | Attack pattern window |
| **rate_limit** (1h) | All others | Per-hour throttle (standard) |

---

## Severity Levels by Sensor

| Severity | Sensors / Events |
|----------|-----------------|
| **critical** | signal_integrity_mapper (orphaned modules), file_hash_watcher (IOC match) |
| **high** | dll_injection_monitor (ld_injection, rwx_memory), entropy_exec_monitor (exec_from_writable), lkm_monitor (suspicious module name), mem_scan (rwx/wx), mfa_bypass, password_spray, suspicious_ipc (world_writable), usb_monitor (vendor-specific/storage) |
| **medium** | cloud_tracker, container_monitor, encrypted_payload_detector (high_entropy), entropy_exec_monitor, file_hash_watcher (modified), geo_ip_anomaly (connection_anomaly), job_sched_monitor, lkm_monitor (license tamper), logon_tracker (system user), mfa_bypass (weak ssh), script_monitor, suspicious_ipc (suspicious_shm), usb_monitor (HID/hub) |
| **info** | All remaining events, telemetry_fingerprint (implied) |

---

## Compilation & Testing

```bash
# Compile all sensors
cargo check --lib

# Run tests
cargo test --lib sensors_v2

# Enable demo mode
export EDR_DEMO=1
cargo run --example edr_demo
```

---

## Implementation Notes

✅ **All sensors are production-ready**:
- Graceful error handling (no panics)
- Bounded memory usage
- Rate-limiting prevents spam
- Read-only (except replay_writer which explicitly avoids file writes)

✅ **No external dependencies needed**:
- Uses only std::fs, std::process, serde_json
- No async I/O (synchronous collection)

✅ **No telemetry/GNN/trust interference**:
- Pure event emission
- No side effects beyond rate-limit tracking
- Fully isolated from other system components

Generated: December 22, 2024
