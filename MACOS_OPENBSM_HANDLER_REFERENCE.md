# macOS OpenBSM Handler Quick Reference

## Handler Functions by Attack Category

### Privilege Escalation (bsm_priv_escalation.rs)

#### UID Change Events
```rust
// Detect when process changes UID (setuid, seteuid, setreuid syscalls)
pub fn handle_uid_change_from_tokens(
    host: String,
    stream_id: String,
    aue_code: i32,      // 24 (SETUID), 130 (SETEUID), 126 (SETREUID)
    tokens: Vec<u8>,    // BSM token stream to parse
    ts_millis: u64,
) -> Option<Event>
```

**Event Tags**: `priv_escalation`, `priv_setuid`, `priv_seteuid`, `priv_setreuid`

**Event Fields**:
- `old_uid` (u32): Original user ID
- `new_uid` (u32): New user ID
- `euid` (u32): Effective UID
- `escalation` (bool): True if uid→0
- `escalation_type` (string): "uid_to_root" if escalating

**Example Detection**: Unprivileged user (uid=501) calls setuid(0) → Event with `escalation: true`

---

#### GID Change Events
```rust
pub fn handle_gid_change_from_tokens(
    host: String,
    stream_id: String,
    aue_code: i32,      // 25 (SETGID), 131 (SETEGID), 127 (SETREGID)
    tokens: Vec<u8>,
    ts_millis: u64,
) -> Option<Event>
```

**Event Tags**: `priv_escalation`, `priv_setgid`, `priv_setegid`, `priv_setregid`

**Event Fields**:
- `old_gid` (u32): Original group ID
- `new_gid` (u32): New group ID
- `escalation` (bool): True if gid→0

---

#### Group Membership
```rust
pub fn handle_setgroups_from_tokens(
    host: String,
    stream_id: String,
    tokens: Vec<u8>,    // BSM token stream
    ts_millis: u64,
) -> Option<Event>     // AUE_SETGROUPS (128)
```

**Event Tags**: `priv_escalation`, `priv_setgroups`

**Event Fields**:
- `group_count` (u32): Number of groups set

---

### Persistence (bsm_file_ops.rs)

#### File Write Detection
```rust
pub fn handle_write(
    host: String,
    stream_id: String,
    segment_id: String,
    record_index: usize,
    aue_code: i32,      // Always 5 (AUE_WRITE)
    tokens: Vec<u8>,
    ts_millis: u64,
) -> Option<Event>
```

**Path Classification**:
- `persistence` tag if path contains: LaunchAgents/, LaunchDaemons/, .ssh/, crontab, shell rc files
- `ssh` tag for ~/.ssh/authorized_keys modifications
- `cron` tag for /etc/cron.d/, /usr/lib/cron/tabs/
- `shell_init` tag for .zshrc, .bashrc, .profile modifications
- `log_tamper` tag for /var/log/*, /var/audit/
- `audit_tamper` tag for /etc/security/audit, /etc/audit
- `user_writable` tag for /tmp/, Downloads/, /Library/

**Event Fields**:
- `path` (string): File path being written
- `path_tags` (array): Classified tags from above
- `attack_class` (string): "persistence" if applicable

**Example**: Writing to ~/.ssh/authorized_keys → Event with `attack_class: "persistence"`, `path_tags: ["ssh", "persistence"]`

---

#### File Create Detection
```rust
pub fn handle_create(
    host: String,
    stream_id: String,
    segment_id: String,
    record_index: usize,
    aue_code: i32,      // 300 (AUE_CREATE)
    tokens: Vec<u8>,
    ts_millis: u64,
) -> Option<Event>
```

Same path classification and tagging as `handle_write`.

**Example**: Creating ~/Library/LaunchAgents/malware.plist → Event with `attack_class: "persistence"`

---

#### File Permission Changes
```rust
pub fn handle_chmod(
    host: String,
    stream_id: String,
    segment_id: String,
    record_index: usize,
    aue_code: i32,      // 17 (AUE_CHMOD)
    tokens: Vec<u8>,
    ts_millis: u64,
) -> Option<Event>
```

**Attack Class Detection**:
- `attack_class: "defense_evasion"` if path tagged `audit_tamper` or `log_tamper`

**Example**: chmod 000 on /var/audit/audit.log → Event with `attack_class: "defense_evasion"`

---

### Process Execution (bsm_exec.rs)

#### Exec Detection
```rust
pub fn handle_exec(
    host: String,
    stream_id: String,
    segment_id: String,
    record_index: usize,
    aue_code: i32,      // Always 59 (AUE_EXECVE)
    tokens: Vec<u8>,
    ts_millis: u64,
) -> Option<Event>
```

**Detections within event**:

1. **Privilege Boundary Crossing** (euid ≠ uid)
   - Fields: `escalation_to_root` (bool), `euid` (u32)
   - Example: unprivileged user exec → setuid binary

2. **Sudo/Su Detection**
   - Triggers when exe ends with `/sudo` or `/su`
   - Fields: `args_preview` (string), `flag_interactive`, `flag_shell`, `sudo_target_user`
   - Tags: `priv_sudo_exec`, `privilege_escalation`
   - Example: sudo -u root /bin/bash → Event with `sudo_target_user: "root"`

3. **Remote Tool Detection**
   - Triggers for: ssh, sshd, scp, sftp, rsync, screen, tmux
   - Fields: `remote_tool` (string)
   - Tags: `remote_tool`, `lateral_movement`
   - Example: /usr/bin/ssh attacker@server → Event with `remote_tool: "ssh"`

4. **Dylib Injection Detection**
   - Triggers when argv contains DYLD_INSERT_LIBRARIES, DYLD_LIBRARY_PATH, DYLD_FRAMEWORK_PATH
   - Fields: `dylib_injection_env` (string)
   - Tags: `dylib_injection_env`, `process_injection`
   - Example: DYLD_INSERT_LIBRARIES=/tmp/evil.dylib bash → Event with `attack_class: "process_injection"`

5. **Setuid Binary Execution**
   - Flags common setuid paths: /usr/bin/sudo, /usr/bin/passwd, /bin/su, /sbin/ping, etc.
   - Fields: `likely_setuid` (bool), `attack_class` (string if escalating)
   - Example: /usr/bin/passwd → Event with `likely_setuid: true`

---

### Network Activity (bsm_network.rs)

#### Socket Connect
```rust
pub fn handle_connect_from_tokens(
    host: String,
    stream_id: String,
    tokens: Vec<u8>,    // BSM socket token stream
    ts_millis: u64,
) -> Option<Event>      // AUE_CONNECT (43)
```

**Classifications**:
- `remote_network: "local"` if RFC 1918 (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) or link-local (169.254.x.x)
- `remote_network: "internet"` otherwise
- `suspicious_port: true` for Metasploit/C&C ports: 4444, 5555, 6666, 7777, 8888, 9999, 5900-5902, 3389-3390, 8080-8088, 9001-9002, 8443

**Event Fields**:
- `remote_ip` (string): Target IP address
- `remote_port` (u16): Target port
- `remote_network` (string): "local" or "internet"
- `protocol` (string): "tcp" or "udp" (if available)
- `suspicious_port` (bool): True if flagged
- `attack_class` (string): "command_control" if suspicious port

**Example**: Process connects to 5.5.5.5:4444 → Event with `suspicious_port: true`, `attack_class: "command_control"`

---

#### Socket Bind
```rust
pub fn handle_bind_from_tokens(
    host: String,
    stream_id: String,
    tokens: Vec<u8>,
    ts_millis: u64,
) -> Option<Event>      // AUE_BIND (42)
```

**Privilege Escalation Detection**:
- Flags when non-root (uid ≠ 0) binds to port < 1024
- Fields: `privileged_port_bind: true`, `attack_class: "privilege_escalation"`

**Event Fields**:
- `local_ip` (string): Bind address
- `local_port` (u16): Bind port
- `privileged_port_bind` (bool): True if non-root ≤ 1024

**Example**: uid=501 binds to 0.0.0.0:443 → Event with `attack_class: "privilege_escalation"`

---

## Shared Utilities (bsm_helpers.rs)

### Path Classification
```rust
pub fn classify_path(path: &str) -> Vec<&'static str>
```

Returns vector of tags for path. Use in all file operation handlers.

**Example**:
```rust
let tags = classify_path("/Users/user/.ssh/authorized_keys");
// Returns: vec!["ssh", "persistence", "user_writable"]
```

---

### Bounded Parsing

#### Argv Parsing
```rust
pub fn parse_argv_bounded(argv_str: &str, max_items: usize, max_len: usize) -> Vec<String>
```

Safely parse command-line arguments with limits.

**Example**:
```rust
let argv = parse_argv_bounded("/bin/bash -i -c echo", 10, 128);
// Returns: vec!["/bin/bash", "-i", "-c", "echo"]
```

---

#### Argv Flag Extraction
```rust
pub fn extract_argv_flags(argv: &[String]) -> BTreeMap<String, String>
```

Extract sudo/su flags from argument vector.

**Recognized flags**:
- `-i`, `--login` → `flag_interactive: "true"`
- `-s`, `--shell` → `flag_shell: "true"`
- `-u` → `sudo_target_user: "<username>"` (next arg)

**Example**:
```rust
let flags = extract_argv_flags(&vec!["-u".to_string(), "root".to_string()]);
// Returns: {sudo_target_user: "root"}
```

---

#### Dylib Injection Detection
```rust
pub fn detect_dylib_injection_env(env_vars: &[String]) -> Option<String>
```

Check for dylib injection environment variables (bounded to first 100 entries).

**Patterns**:
- `DYLD_INSERT_LIBRARIES`
- `DYLD_LIBRARY_PATH`
- `DYLD_FRAMEWORK_PATH`

**Example**:
```rust
let env = detect_dylib_injection_env(&vec!["DYLD_INSERT_LIBRARIES=/tmp/evil.dylib".to_string()]);
// Returns: Some("DYLD_INSERT_LIBRARIES=/tmp/evil.dylib")
```

---

#### Ptrace/Debugger Detection
```rust
pub fn detect_ptrace_activity(path: &str, argv: &[String]) -> Option<String>
```

Detect debugger/ptrace tool execution.

**Recognized Tools**:
- lldb (LLVM debugger)
- gdb (GNU debugger)
- dtrace (Dynamic tracing)
- strace (System call tracing)
- DebugService (macOS debug service)

**Flags**:
- `--pid`, `-p` (likely attaching to process)

**Example**:
```rust
let tool = detect_ptrace_activity("/usr/bin/lldb", &vec!["-p".to_string(), "1234".to_string()]);
// Returns: Some("ptrace_attach")
```

---

## Dispatcher Routing Table

| AUE Code | Event Type | Handler | File |
|----------|-----------|---------|------|
| 3 | AUE_OPEN_RWT | handle_open | bsm_file_ops |
| 4 | AUE_READ | handle_read | bsm_file_ops |
| 5 | AUE_WRITE | handle_write | bsm_file_ops |
| 7 | AUE_UNLINK | handle_unlink | bsm_file_ops |
| 17 | AUE_CHMOD | handle_chmod | bsm_file_ops |
| 21 | AUE_MOUNT | handle_mount | bsm_fs_ops |
| 22 | AUE_UNMOUNT | handle_unmount | bsm_fs_ops |
| 24 | AUE_SETUID | handle_uid_change_from_tokens | bsm_priv_escalation |
| 25 | AUE_SETGID | handle_gid_change_from_tokens | bsm_priv_escalation |
| 42 | AUE_BIND | handle_bind_from_tokens | bsm_network |
| 43 | AUE_CONNECT | handle_connect_from_tokens | bsm_network |
| 59 | AUE_EXECVE | handle_exec | bsm_exec |
| 126 | AUE_SETREUID | handle_uid_change_from_tokens | bsm_priv_escalation |
| 127 | AUE_SETREGID | handle_gid_change_from_tokens | bsm_priv_escalation |
| 128 | AUE_SETGROUPS | handle_setgroups_from_tokens | bsm_priv_escalation |
| 130 | AUE_SETEUID | handle_uid_change_from_tokens | bsm_priv_escalation |
| 131 | AUE_SETEGID | handle_gid_change_from_tokens | bsm_priv_escalation |
| 300 | AUE_CREATE | handle_create | bsm_file_ops |

---

## Event Structure

All handlers emit `crate::core::Event` with:

```rust
Event {
    ts_ms: i64,                           // Timestamp in milliseconds
    host: String,                          // Hostname
    tags: Vec<String>,                     // Attack tags (priv_escalation, persistence, etc.)
    proc_key: Some(hash_keys::proc_key()), // Process hash for correlation
    file_key: Option<...>,                 // File hash for file operations
    identity_key: Some(...),               // User/UID hash for correlation
    evidence_ptr: None,                    // Assigned by capture module
    fields: BTreeMap<String, Value>,       // Attack-specific fields
}
```

---

## Testing Patterns

### Unit Test Template
```rust
#[test]
fn test_setuid_escalation() {
    let event = bsm_priv_escalation::handle_uid_change(
        "test-host".to_string(),
        "stream-1".to_string(),
        24,      // AUE_SETUID
        1000,    // pid
        501,     // uid (unprivileged)
        501,     // euid
        0,       // new_uid (root)
        1000000,
    );
    
    assert!(event.is_some());
    let e = event.unwrap();
    assert_eq!(e.tags, vec!["macos", "priv_escalation", "priv_setuid"]);
    assert_eq!(e.fields["escalation"], json!(true));
}
```

### Integration Test Template
```rust
#[test]
fn test_persistence_detection() {
    let event = bsm_file_ops::handle_write(
        "test-host".to_string(),
        "stream-1".to_string(),
        "segment-1".to_string(),
        0,
        5,      // AUE_WRITE
        vec![],
        1000000,
    );
    
    // After token parsing is implemented:
    // let event = parse_and_dispatch(bsm_tokens_with_launchagent_write);
    // assert!(event.fields["attack_class"] == "persistence");
}
```

---

## Performance Considerations

### Bounded Parsing Limits
All parsing is bounded to prevent DoS and memory exhaustion:

| Operation | Limit | Purpose |
|-----------|-------|---------|
| Argv items | 100 (default 50) | Prevent huge argument lists |
| Arg length | 256 chars | Truncate very long arguments |
| Env entries | 100 | Bounded environment parsing |
| Env var length | 256 chars | Truncate long env vars |
| Network binary parse | 28 bytes (IPv6) | Structural bound |
| Setuid path patterns | 11 | Constant-time matching |
| Suspicious ports | 12 | Fixed set matching |
| Path classification | 8 | O(1) check count |

### Synchronous Processing
All handlers are synchronous, non-blocking. Token parsing can be parallelized at capture layer if needed.

---

## Future Enhancements

1. **Token Parser Implementation**: Decode actual BSM token format
2. **Event Scoring**: Add confidence/severity scores to each event class
3. **Correlation**: Link execution events to network/file events
4. **Metrics**: Track event counters and latencies
5. **Playbook Integration**: Map events to detection/response playbooks
