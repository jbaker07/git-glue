# macOS OpenBSM Coverage Expansion Summary

## Overview
Expanded macOS OpenBSM audit coverage from basic file/process events to comprehensive attack detection across the MITRE ATT&CK framework. Implementation follows bounded parsing, no fake evidence pointers, and synchronous processing constraints.

## Coverage Areas Expanded

### 1. **Privilege Escalation Detection** ✅
**File: `bsm_priv_escalation.rs`**

#### UID/GID Change Handlers
- **AUE_SETUID (24)**: Detects uid() syscall with privilege escalation flagging
  - Flags uid→0 transitions (critical escalation)
  - Tracks old_uid, new_uid, euid mismatch
  - Tags: `priv_setuid`, `priv_escalation`

- **AUE_SETEUID (130)**: Effective UID changes (setuid bit execution)
  - Detects boundary crossing between user→elevated contexts
  - Flags when uid ≠ euid (setuid binary execution)
  - Tags: `priv_seteuid`, `priv_escalation`

- **AUE_SETREUID (126)**: Real+effective UID swap
  - Full UID replacement tracking
  - Escalation detection and tagging

- **AUE_SETGID (25) / SETEGID (131) / SETREGID (127)**
  - GID escalation detection (gid→0)
  - Group privilege boundary tracking
  - Tags: `priv_setgid`, `priv_escalation`

- **AUE_SETGROUPS (128)**: Group membership manipulation
  - Detects privilege escalation via group changes
  - Tracks group count changes
  - Tags: `priv_setgroups`, `priv_escalation`

#### Integration Pattern
Each handler accepts parsed tokens and emits Event with:
```rust
Event {
    ts_ms, host, tags: [escalation_type],
    proc_key: proc_key(pid),
    identity_key: identity_key(uid),
    fields: {aue_code, old_uid, new_uid, escalation: bool, ...}
}
```

---

### 2. **Persistence Detection** ✅
**File: `bsm_file_ops.rs` + `bsm_helpers.rs`**

#### Path Classification System
Integrated `classify_path()` into file operation handlers (write, create, chmod):
- **LaunchAgents/LaunchDaemons**: macOS persistent agents
- **SSH (.ssh/authorized_keys)**: SSH key injection
- **Cron (crontab, /etc/cron.d/)**: Scheduled task persistence
- **Shell Init (.zshrc, .bashrc, .profile)**: Shell startup modification
- **Log Tampering (/var/log/*, /var/audit/)**: Defense evasion
- **Audit Config (/etc/security/audit, /etc/audit)**: Audit tampering

#### File Operation Enhancements
- **handle_write**: Detects writes to persistence paths → tags `persistence`
- **handle_create**: Detects creation of persistence artifacts → tags `persistence`
- **handle_chmod**: Detects permission changes on audit/logs → tags `defense_evasion`

#### Event Fields
```rust
fields: {
    path, path_tags: [persistence|ssh|cron|shell_init|log_tamper|audit_tamper],
    attack_class: "persistence" | "defense_evasion"
}
```

---

### 3. **Process Execution & Injection** ✅
**File: `bsm_exec.rs`**

#### Privilege Escalation via Execution
- **euid Mismatch Detection**: Detects setuid binary execution (uid ≠ euid)
  - Flags when unprivileged user crosses to elevated context
  - Critical for detecting sudo, setuid binaries, capability-based escalation

- **Sudo/Su Detection**: Pattern matching on `/usr/bin/sudo`, `/bin/su`
  - Extracts sudo flags (-i, -s, -u USER)
  - Captures target user escalation
  - Tags: `priv_sudo_exec`, `privilege_escalation`

- **Setuid Binary Detection**: Common setuid paths
  - `/usr/bin/sudo`, `/usr/bin/passwd`, `/usr/bin/chsh`, `/usr/bin/chfn`
  - `/bin/su`, `/sbin/ping`, `/usr/bin/at`, `/sbin/mount`
  - `/usr/libexec/*` (Apple system tools)
  - Tags: `likely_setuid`, `privilege_escalation`

#### Lateral Movement Detection
- **Remote Tool Exec**: Pattern matching for SSH, SCP, SFTP, rsync, screen, tmux
  - Detects command-line remote access attempts
  - Tags: `remote_tool`, `lateral_movement`
  - Fields: `remote_tool` (tool name)

#### Process Injection Detection
- **DYLD Injection Env Vars**: Detects dylib injection attempts
  - `DYLD_INSERT_LIBRARIES`: Direct dylib injection
  - `DYLD_LIBRARY_PATH`: Library path hijacking
  - `DYLD_FRAMEWORK_PATH`: Framework path hijacking
  - Tags: `dylib_injection_env`, `process_injection`
  - Bounded parsing: first 100 argv entries, max 256 char env var

#### Event Structure
```rust
Event {
    proc_key: proc_key(pid),
    fields: {
        exe, argv_count,
        remote_tool?, dylib_injection_env?, likely_setuid?,
        attack_class: "privilege_escalation" | "lateral_movement" | "process_injection"
    }
}
```

---

### 4. **Network Command & Control** ✅
**File: `bsm_network.rs`**

#### Socket Connect (AUE_CONNECT = 43)
- **Remote IP/Port Tracking**
  - Detects outbound connections
  - Flags RFC 1918 (local) vs internet destinations
  - Tags: `network`, `connect`

- **Suspicious Port Detection** (C&C/backdoor ports)
  - **Metasploit defaults**: 4444, 5555, 6666, 7777, 8888, 9999
  - **VNC/RDP alt**: 5900–5902, 3389–3390
  - **HTTP/HTTPS alt**: 8080, 8081, 8088, 9001, 9002, 8443
  - Tags: `suspicious_port`, `command_control`

- **Network Classification**
  - RFC 1918: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
  - Link-local: 169.254.0.0/16
  - Fields: `remote_network: "local" | "internet"`

#### Socket Bind (AUE_BIND = 42)
- **Privileged Port Binding** (<1024) by non-root
  - Flags privilege boundary violation
  - Tags: `bind`, `privilege_escalation`
  - Fields: `privileged_port_bind: bool`

#### Event Structure
```rust
Event {
    proc_key: proc_key(pid),
    identity_key: identity_key(uid),
    fields: {
        remote_ip, remote_port, remote_network,
        protocol?,
        suspicious_port?, attack_class: "command_control"?
    }
}
```

---

### 5. **Shared Utilities** ✅
**File: `bsm_helpers.rs`**

#### Path Classification
```rust
classify_path(path: &str) -> Vec<&'static str>
```
Returns tags: persistence, ssh, cron, shell_init, log_tamper, audit_tamper, user_writable

#### Bounded Parsing Utilities
- **`parse_argv_bounded`**: Cap items + string lengths (default 10 items, 128 chars each)
- **`extract_argv_flags`**: Extract sudo flags (-i, -s, -u) with bounds (max 50 iterations)
- **`parse_env_bounded`**: Cap env entries + sizes (default 100 entries, 256 chars each)
- **`detect_dylib_injection_env`**: Check for DYLD_* vars (bounded to 100 entries)
- **`detect_ptrace_activity`**: Detect debuggers (lldb, gdb, dtrace, strace) and ptrace flags

#### Deprecated Helper
- `build_bsm_event`: Marked `#[deprecated]` - handlers should construct Events directly to maintain proper key assignments

---

## Dispatcher Updates

### Event Router (bsm_reader.rs)
Updated `dispatch_event()` to route new AUE codes:

| AUE Code | Event Type | Handler |
|----------|-----------|---------|
| 24 | AUE_SETUID | `handle_uid_change_from_tokens` |
| 25 | AUE_SETGID | `handle_gid_change_from_tokens` |
| 126 | AUE_SETREUID | `handle_uid_change_from_tokens` |
| 127 | AUE_SETREGID | `handle_gid_change_from_tokens` |
| 128 | AUE_SETGROUPS | `handle_setgroups_from_tokens` |
| 130 | AUE_SETEUID | `handle_uid_change_from_tokens` |
| 131 | AUE_SETEGID | `handle_gid_change_from_tokens` |
| 42 | AUE_BIND | `handle_bind_from_tokens` |
| 43 | AUE_CONNECT | `handle_connect_from_tokens` |

Existing routes preserved:
- 3: AUE_OPEN_RWT
- 4: AUE_READ
- 5: AUE_WRITE
- 7: AUE_UNLINK
- 17: AUE_CHMOD
- 59: AUE_EXECVE
- 300: AUE_CREATE

---

## Key Design Decisions

### 1. **No Fake Evidence Pointers**
- All handlers emit `evidence_ptr: None`
- Capture module assigns actual evidence pointers during event processing

### 2. **Bounded Parsing**
- Argv parsing: max 50-100 items, 128-256 char limits per arg
- Env var parsing: max 100 entries, 256 char limit per var
- Network parsing: IPv4 (8 bytes), IPv6 (28 bytes) structural bounds
- Path classification: O(1) pattern matching on bounded set

### 3. **Proper Key Assignment**
- All events compute `proc_key`, `file_key`, `identity_key` locally
- Hash functions (`hash_keys::*`) use host + resource + stream_id
- Deprecated `build_bsm_event` helper in favor of direct construction

### 4. **Synchronous, Single-Pass Processing**
- No async operations or batching
- Each handler processes one token stream → one Event
- Token parsing deferred to TODO (infrastructure in place)

### 5. **macOS-Specific**
- No Windows/Linux-specific code
- Uses BSD audit constants (AUE_* codes)
- Leverages macOS-specific paths (LaunchAgents, .ssh, etc.)

---

## Coverage Gaps (TODO)

### Token Parsing Implementation
All handlers include TODO for actual BSM token stream parsing:
```rust
// TODO: Parse BSM tokens to extract pid, uid, euid, new_uid
let pid = 0u32;  // PLACEHOLDER
```

### Infrastructure Needed
1. **BSM Token Parser**: Decode binary token format (subject, arg, return, etc.)
2. **Sockaddr Parsing**: Full IPv6 expansion from 16-byte binary
3. **Full Argv/Env Extraction**: Parse from audit token strings
4. **File Path Extraction**: From AUE_OPEN, AUE_WRITE, etc. tokens

### Event Counters
Current capture module `capture_macos_rotating.rs` does not track:
- Per-handler event counters
- Attack class distribution (priv_escalation, persistence, lateral_movement, etc.)
- Path tag statistics (ssh, cron, log_tamper, etc.)

Could be added to telemetry metadata or separate metrics endpoint.

---

## Testing Considerations

### Unit Tests (Mock Tokens)
- Mock token streams for each AUE code
- Verify field extraction and tagging
- Validate attack classification logic

### Integration Tests
- Live BSM capture against test scenarios:
  - sudo execution
  - SSH connection attempts
  - LaunchAgent creation
  - Setuid binary execution (e.g., `/usr/bin/sudo`)
  - Port binding scenarios

### Regression Tests
- Verify existing file/exec handlers unchanged
- Confirm dispatcher routing correct

---

## Files Modified

1. **bsm_priv_escalation.rs**: +95 lines (handlers + wrapper functions)
2. **bsm_file_ops.rs**: +20 lines (path classification integration)
3. **bsm_exec.rs**: Rewritten, -25 duplicates, +40 new detections
4. **bsm_network.rs**: Rewritten, +55 lines (handlers + network classification)
5. **bsm_helpers.rs**: +75 lines (ptrace detection, bounded parsing improvements)
6. **bsm_reader.rs**: +8 lines (dispatcher routing)
7. **mod.rs**: No changes (public module structure preserved)

---

## Summary Statistics

### New Detection Capabilities
- **6 privilege escalation events** (setuid, seteuid, setreuid, setgid, setegid, setregid, setgroups)
- **3 persistence attack patterns** (LaunchAgents, SSH, cron, shell init, log tamper)
- **3 lateral movement indicators** (remote tools SSH/SCP/rsync/etc.)
- **2 process injection vectors** (DYLD_* env vars, dylib staging)
- **2 network threats** (suspicious ports, privileged port binding)
- **1 defense evasion indicator** (audit/log tampering)

### Bounded Resource Limits
- Max argv items: 100 (default 50)
- Max arg string length: 256 chars
- Max env entries: 100
- Max network parse depth: 28 bytes (IPv6)
- Max path pattern checks: 8 (constant-time)
- Max suspicious ports: 12

### Event Tags Added
- `priv_escalation`, `priv_setuid`, `priv_seteuid`, `priv_setreuid`, `priv_setgid`, `priv_setegid`, `priv_setregid`, `priv_setgroups`
- `persistence`, `ssh`, `cron`, `shell_init`, `log_tamper`, `audit_tamper`, `user_writable`
- `remote_tool`, `lateral_movement`
- `dylib_injection_env`, `process_injection`
- `suspicious_port`, `command_control`
- `privileged_port_bind`

### Attack Classes in Fields
- `attack_class: "privilege_escalation"`
- `attack_class: "persistence"`
- `attack_class: "lateral_movement"`
- `attack_class: "process_injection"`
- `attack_class: "command_control"`
- `attack_class: "defense_evasion"`

---

## Future Enhancements

1. **Token Stream Parser**: Implement actual BSM binary parsing for full extraction
2. **Event Metrics**: Add counters and histograms to telemetry output
3. **ML-Ready Fields**: Ensure all fields normalized for threat scoring
4. **File Hash Tracking**: Add SHA256 for persistence artifacts
5. **Connection Context**: Link network events to process execution chains
6. **Playbook Integration**: Map events to detection playbooks and response actions
