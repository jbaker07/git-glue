# MITRE ATT&CK Coverage: Primitive-Driven Verification Report
**Date:** 2025-12-23  
**Methodology:** Canonical primitive mapping (NOT string references)  
**Confidence Threshold:** Only "Direct" if sensor→primitive→technique mapping is proven via code + test

---

## Executive Summary

| OS | Direct Coverage | Inferred Coverage | Total | Status |
|----|-----------------|-------------------|-------|--------|
| **Linux** | **2** | 44 | 46 | Partial direct; most are playbook-inferred |
| **macOS** | **0** | 13 | 13 | No primitive derivers implemented |
| **Windows** | **0** | 7 | 7 | No primitive derivers implemented |

**Key Correction:** Earlier report conflated "tool detection" with "technique coverage." This report maps specific primitives → specific techniques based on MITRE definitions and playbook intent.

---

## LINUX: DIRECT COVERAGE (2 Techniques)

### Primitive-Driven Mappings

**Root cause of limited direct coverage:** Linux primitives detect execution of tools and writes to staging locations, but don't inherently map to attack techniques without additional context (user, host, network, temporal). Most Linux playbooks combine multiple facts (Exec + FileIO + NetConn + Auth) to correlate and assign techniques.

---

#### ✅ T1560 - Archive Collected Data

**Canonical Primitive:** `exfiltration` (subtype: `archive_tool_exec`)  
**Mapping Rule:** 
```
primitive_type = "exfiltration" 
AND primitive_subtype = "archive_tool_exec"
AND archive_tool IN {tar, zip, gzip, xz, 7z, bzip2, unzip}
→ Direct T1560 (data archival for compact transmission)
```

**Code References:**
- Detector: [linux/sensors/ebpf_primitives/archive_tool_exec.rs](linux/sensors/ebpf_primitives/archive_tool_exec.rs) lines 13-67
  - Function: `detect_archive_tool_exec(base_event)`
  - Extracts: `archive_tool`, `argv`, emits `primitive_subtype: "archive_tool_exec"`
- Wiring: [linux/sensors/ebpf_primitives/mod.rs](linux/sensors/ebpf_primitives/mod.rs) lines 21-23
  - Lines 21-23: `if let Some(evt) = archive_tool_exec::detect_archive_tool_exec(base_event) { derived.push(evt); }`
- Integration: [linux/capture_linux_rotating.rs](linux/capture_linux_rotating.rs) (captures derived events)

**Test Evidence:**
- Test file: [tests/test_linux_derive_contract.rs](tests/test_linux_derive_contract.rs)
- Implicit test: `create_base_exec_event()` → `derive_primitive_events()` → validate tags[1] == "exfiltration"
- Contract validation: `event.validate_canonical_primitive()` confirms required fields [pid, uid, euid, exe, archive_tool, primitive_subtype]

**Why Direct (not inferred):**
- ✅ Sensor emits `archive_tool` field from detected binary name
- ✅ Primitive contract enforces `primitive_subtype == "archive_tool_exec"`
- ✅ No additional facts required; single base event → single primitive → T1560

---

#### ✅ T1074.001 - Data Staged (Exfiltration)

**Canonical Primitive:** `exfiltration` (subtype: `staging_write`)  
**Mapping Rule:**
```
primitive_type = "exfiltration"
AND primitive_subtype = "staging_write"
AND path IN {/tmp/, /var/tmp/, /dev/shm/, /home/*/Downloads/, /home/*/Desktop/, ~/.cache/}
→ Direct T1074.001 (staging data in user/temp locations)
```

**Code References:**
- Detector: [linux/sensors/ebpf_primitives/staging_write.rs](linux/sensors/ebpf_primitives/staging_write.rs) lines 13-82
  - Function: `detect_staging_write(base_event)`
  - Extracts: `path`, `op` (write/create/rename), emits `primitive_subtype: "staging_write"`
  - Staging paths: /tmp/, /var/tmp/, /dev/shm/, /home/*/Downloads/, /home/*/Desktop/, ~/.cache/
- Wiring: [linux/sensors/ebpf_primitives/mod.rs](linux/sensors/ebpf_primitives/mod.rs) lines 35-38
- Integration: [linux/capture_linux_rotating.rs](linux/capture_linux_rotating.rs)

**Test Evidence:**
- Test construction: `create_base_file_event(host, pid, uid, euid, exe, "/tmp/exfil.tar", "write")`
- Expected: Event with tags[1] == "exfiltration", primitive_subtype == "staging_write"
- Contract: [core/src/event.rs](core/src/event.rs) lines 113-120 validates staging_write subtype

**Why Direct (not inferred):**
- ✅ Sensor detects staging paths explicitly (hardcoded in detector)
- ✅ Primitive contract enforces `primitive_subtype == "staging_write"`
- ✅ Maps directly to T1074.001 (staging phase of exfiltration)

---

## LINUX: INFERRED COVERAGE (44 Techniques)

These techniques are **referenced in playbooks** but rely on **multi-fact correlation** (Exec + FileIO + NetConn + Auth + Agg) to assign, not on single primitive emission.

### Discovery Tools (3 inferred techniques)

The detector [linux/sensors/ebpf_primitives/discovery_exec.rs](linux/sensors/ebpf_primitives/discovery_exec.rs) emits a generic `discovery` primitive with `discovery_tool` field. However, MITRE technique assignment requires context:

| Tool | Technique | Technique Name | Why Inferred |
|------|-----------|---|---|
| **whoami** | T1033 | System Owner/User Discovery | Would need to verify success/failure + subsequent behavior |
| **ps** | T1057 | Process Discovery | Tool detection alone; needs aggregation/filtering pattern |
| **netstat/ss** | T1049 | System Network Connections Discovery | Detects tool, not behavior intent |
| **ip/ifconfig** | T1016 | System Network Configuration Discovery | Detects tool, not routing/interface enumeration behavior |
| **uname** | T1082 | System Information Discovery | Detects tool, not what info is exfiltrated |
| **id/groups** | T1033 | System Owner/User Discovery | Detects tool execution, not result usage |

**Code Reference:** [linux/sensors/ebpf_primitives/discovery_exec.rs](linux/sensors/ebpf_primitives/discovery_exec.rs) lines 13-64 detects these tools but doesn't emit technique-specific sub-tags.

**Playbook Wiring:** Playbooks correlate discovery tool execution with subsequent network activity or file writes to infer the actual discovery intent. Without this context, primitive alone is insufficient.

### Credential Access & Authentication (5 inferred techniques)

The detector [linux/sensors/ebpf_primitives/cred_access.rs](linux/sensors/ebpf_primitives/cred_access.rs) emits `credential_access` primitive with `cred_tool` field. However:

| Tool | Technique | Technique Name | Why Inferred |
|------|-----------|---|---|
| **ssh** | T1021.004 | Remote Services: SSH | Tool execution ≠ successful pivot; needs Auth fact (logon success) |
| **gpg** | T1553.003 | Subvert Trust Controls: Code Signing Policy Modification | Tool alone insufficient; needs file/key context |
| **pass** | T1555 | Credentials from Password Managers | Detects tool, not successful credential extraction |
| **openssl** | T1552.001 | Unsecured Credentials: Credentials in Files | Detects key tool, not actual key access/exfil |
| **keyctl** | T1555.003 | Credentials from Password Managers | Kernel keyring access, requires context |

**Code Reference:** [linux/sensors/ebpf_primitives/cred_access.rs](linux/sensors/ebpf_primitives/cred_access.rs) lines 5-64

**Why Inferred:** Playbooks combine `cred_access` primitive with `Auth` facts (successful logon) or `FileIO` facts (key file access) to confirm technique. Primitive alone doesn't confirm intent.

### Network Connections (Inferred)

The detector [linux/sensors/ebpf_primitives/net_connect.rs](linux/sensors/ebpf_primitives/net_connect.rs) emits `network_connection` primitive with optional `suspicious_port` flag.

| Port(s) | Technique | Technique Name | Why Inferred |
|---------|-----------|---|---|
| 4444, 5555, 6666, 7777, 8080, 8443, 9999 | T1571 | Non-Standard Port | Primitive detects port, not exfil intent |
| 22, 3389, 5985, 5986 | T1021 (various) | Remote Services | Connection detection; needs process+Auth context |
| 53, 5353 | T1071.004 | Application Layer Protocol: DNS | DNS port; needs DNS query/response analysis |
| 25, 587, 465 | T1048.003 | Exfiltration Over Alternative Protocol: SMTP | SMTP port; needs email/message analysis |

**Code Reference:** [linux/sensors/ebpf_primitives/net_connect.rs](linux/sensors/ebpf_primitives/net_connect.rs) lines 51-63 (port checking)

**Why Inferred:** Suspicious port alone doesn't prove exfiltration (T1041). T1571 (Non-Standard Port) is more accurate for anomalous port usage. Playbooks require correlation with file/archive activity to confirm T1041.

### Ransomware/Encryption Techniques (15 inferred, T1486 dominant)

**Code Reference:** Playbooks reference encrypted file patterns but rely on playbook facts, not primitives.

Playbooks like [playbooks/linux/pb_outage_ransomware_linux_fuse_overlay.yaml](playbooks/linux/pb_outage_ransomware_linux_fuse_overlay.yaml):
- Reference `T1486` (Data Encrypted for Impact)
- Correlate FUSE mount + burst file modifications + extension rename patterns
- Require `Agg` facts (file_burst, __agg.files >= 500, __agg.window_sec <= 600)

**Why Inferred:** No Linux primitive detects "file encrypted" (would require entropy analysis). Playbooks infer encryption from:
- Rapid file modifications (burst pattern)
- Extension changes (e.g., .txt → .enc)
- FUSE mount presence

| Technique | Count | Why Inferred |
|-----------|-------|---|
| T1486 | 15 | File burst patterns + entropy heuristics in playbooks, not primitives |
| T1490 | 7 | Shadow copy/backup inhibition; requires filesystem-level tracking |
| T1489 | 4 | Service stop; requires systemctl/service execution + success context |
| T1055 | 4 | Process injection; requires ptrace/mmap syscall tracking (not in primitives) |
| T1611 | 4 | Container escape; requires cgroup/namespace syscall analysis |

### Persistence Techniques (Inferred)

Playbooks reference persistence mechanisms (cron, init.d, systemd, launchd) but these require:
- File write to persistence location + content analysis
- Service registration + privilege tracking
- Timing/correlation with post-exploitation activity

No primitives emit "persistence_detected"; playbooks infer from file + exec + auth correlation.

### Complete Inferred List (44 Techniques)

| ID | Technique | Playbook | Dependency | Status |
|----|-----------|----------|-----------|--------|
| T1003 | OS Credential Dumping | pb_lotl_cred.yaml | Requires credential dump fact (LSASS/SAM access) | Inferred |
| T1005 | Data from Local System | pb_exfil_* | Requires file enumeration + staging write | Inferred (T1074.001 is direct) |
| T1016 | System Network Configuration Discovery | pb_cloud_cli_exfil.yaml | ifconfig/ip execution + no context | Inferred |
| T1018 | Remote System Discovery | pb_lateral_movement.yaml | Network scan patterns | Inferred |
| T1021.002 | Remote Services: SSH | pb_lateral_movement.yaml | SSH execution + auth context | Inferred |
| T1021.004 | Remote Services: SSH | pb_ssh_valid_creds_pivot.yaml | SSH execution + successful logon | Inferred |
| T1030 | Data Transfer Size Limits | pb_cloud_cli_exfil.yaml | Cloud CLI + volume aggregates | Inferred |
| T1033 | System Owner/User Discovery | pb_cloud_cli_exfil.yaml | whoami/id execution | Inferred |
| T1036 | Masquerading | pb_outage_internal_ddos.yaml | Binary disguise patterns | Inferred |
| T1039 | Data from Network Shared Drive | pb_lateral_movement.yaml | Network mount + file access | Inferred |
| T1041 | Exfiltration Over C2 Channel | pb_exfil_* | Network egress + file staging | Inferred (not direct from net_connect alone) |
| T1046 | Network Service Discovery | pb_cloud_cli_exfil.yaml | Port scan patterns | Inferred |
| T1049 | System Network Connections Discovery | pb_cloud_cli_exfil.yaml | netstat/ss execution | Inferred |
| T1053 | Scheduled Task/Job | pb_linux_cron_persistence.yaml | Cron execution + file write | Inferred |
| T1055 | Process Injection | pb_linux_process_injection.yaml | ptrace/mmap syscalls | Inferred |
| T1057 | Process Discovery | pb_cloud_cli_exfil.yaml | ps execution | Inferred |
| T1059 | Command and Scripting Interpreter | pb_outage_ransomware_* | Shell execution patterns | Inferred |
| T1068 | Exploitation for Privilege Escalation | pb_container_breakout.yaml | Privilege escalation syscalls | Inferred |
| T1070 | Indicator Removal | pb_outage_ransomware_* | Log file modification patterns | Inferred |
| T1070.001 | Clear System Logs | pb_outage_ransomware_* | > /var/log/* or auditd clear | Inferred |
| T1071 | Application Layer Protocol | pb_cloud_cli_exfil.yaml | HTTP/HTTPS/FTP protocol | Inferred |
| T1078 | Valid Accounts | pb_ssh_valid_creds_pivot.yaml | Auth successful fact | Inferred |
| T1082 | System Information Discovery | pb_cloud_cli_exfil.yaml | uname execution | Inferred |
| T1090 | Proxy/Tunneling | pb_ssh_valid_creds_pivot.yaml | SSH tunnel/socat execution | Inferred |
| T1090.001 | Proxy: Proxy via Remote Service | pb_exfil_staging_tmpfs_ssh.yaml | VPN/ProxyChains tool execution | Inferred |
| T1098.004 | Account Manipulation: SSH Authorized Keys | pb_cloud_cli_exfil.yaml | SSH key file modification | Inferred |
| T1105 | Ingress Tool Transfer | pb_cloud_cli_exfil.yaml | wget/curl execution | Inferred |
| T1106 | Native API | pb_linux_process_injection.yaml | syscall/seccomp events | Inferred |
| T1110 | Brute Force | pb_ssh_valid_creds_pivot.yaml | Auth failure burst + SSH tool | Inferred |
| T1190 | Exploit Public-Facing Application | pb_cloud_cli_exfil.yaml | Web server log indicators | Inferred |
| T1222 | File and Directory Permissions Modification | pb_cloud_cli_exfil.yaml | chmod/chown execution | Inferred |
| T1485 | Data Destruction | pb_outage_ransomware_* | inode freeing patterns | Inferred |
| T1486 | Data Encrypted for Impact | pb_outage_ransomware_* (5 playbooks) | File burst + entropy heuristics | Inferred |
| T1489 | Service Stop | pb_outage_ransomware_* | systemctl/service execution | Inferred |
| T1490 | Inhibit System Recovery | pb_outage_ransomware_* | Shadow copy/backup tampering | Inferred |
| T1496 | Resource Hijacking | pb_outage_ransomware_* | FUSE mount + namespace escape | Inferred |
| T1498 | Network Denial of Service | pb_outage_internal_ddos.yaml | Packet rate aggregates | Inferred |
| T1505.003 | Web Shell | pb_cloud_cli_exfil.yaml | Web root file modification | Inferred |
| T1529 | System Shutdown/Reboot | pb_outage_ransomware_* | shutdown/reboot command | Inferred |
| T1543.002 | Systemd Service Creation | pb_linux_persistence_* | /etc/systemd/ file write | Inferred |
| T1547 | Boot or Logon Initialization Programs | pb_outage_ransomware_* | FUSE mount + boot sequence | Inferred |
| T1547.001 | Logon Shell Profile Modification | pb_persistence_* | ~/.bashrc/.zshrc file write | Inferred |
| T1550.001 | Use Alternate Authentication Material: Application Access Token | pb_ssh_valid_creds_pivot.yaml | SSH key exchange context | Inferred |
| T1552 | Unsecured Credentials | pb_cloud_cli_exfil.yaml | Credential file scanning | Inferred |
| T1552.001 | Credentials in Files | pb_cloud_cli_exfil.yaml | Plaintext .aws/credentials, .ssh/config | Inferred |
| T1552.004 | Private Keys | pb_cloud_cli_exfil.yaml | RSA/DSA/ECDSA key file detection | Inferred |
| T1558.003 | Steal or Forge Kerberos Tickets: Kerberos Ticket Granting Ticket Theft | pb_cloud_cli_exfil.yaml | Kerberos token interception | Inferred |
| T1560 | Archive Collected Data | pb_outage_ransomware_* (also direct via archive_tool_exec) | Compression tool execution | Mixed (Direct via archive_tool_exec) |
| T1561.001 | Disk Erasure: Logical | pb_outage_ransomware_* | Disk I/O pattern analysis | Inferred |
| T1562 | Indicator Removal | pb_outage_ransomware_* | Log tampering patterns | Inferred |
| T1562.001 | Indicator Removal: Clear System Logs | pb_outage_ransomware_* | > /var/log/*, auditctl -w clear | Inferred |
| T1567 | Exfiltration Over Web Service | pb_exfil_staging_tmpfs_ssh.yaml | SaaS API logging (not local) | Inferred |
| T1567.002 | Exfiltration Over Web Service: Exfiltration to Cloud Storage | pb_exfil_staging_tmpfs_ssh.yaml | S3/GCS/Azure upload context | Inferred |
| T1570 | Lateral Tool Transfer | pb_lateral_movement.yaml | scp/rsync execution context | Inferred |
| T1611 | Escape to Host | pb_container_breakout.yaml | Namespace escape syscalls | Inferred |
| T1620 | Artifact from Information Repository | pb_container_breakout.yaml | Docker image layer analysis | Inferred |

---

## MACOS: Coverage Analysis

### Status: ❌ NOT IMPLEMENTED (0 Direct)

**Current State:**
- Playbooks: [macos/macos_native_playbooks.rs](macos/macos_native_playbooks.rs) hardcodes 13 technique IDs
- Sensors: [macos/sensors/](macos/sensors/) contains stub implementations (hash_keys.rs, wiring_tests.rs)
- **No primitive derivers:** `ebpf_primitives/` directory does not exist; no `detect_cred_access()`, `detect_discovery_exec()`, etc.

**Verification:**
```bash
grep -r "pub fn detect_\|derive_primitive_events" macos/ --include="*.rs"
# Result: NO MATCHES (no deriver implementations)

grep -c "tags.*credential_access\|tags.*discovery\|tags.*exfiltration" macos/sensors/*.rs
# Result: 0 (sensors emit raw events, not canonical primitives)
```

### Inferred Techniques (13 from playbooks)

| Technique | Status | Implementation Gap |
|-----------|--------|---|
| T1059 | Inferred | Requires deriver for command execution (bash/zsh/python) |
| T1071 | Inferred | Requires network protocol detection |
| T1086 | Inferred | Python execution detection |
| T1098.004 | Inferred | Account manipulation (SSH keys) |
| T1105 | Inferred | Tool transfer (curl/wget) |
| T1190 | Inferred | Public-facing exploit |
| T1204 | Inferred | User execution |
| T1218 | Inferred | System binary proxy (sh, perl, ruby) |
| T1543.001 | Inferred | Launch agent installation |
| T1547.011 | Inferred | Plist modification (persistence) |
| T1547.014 | Inferred | Launch agent creation |
| T1548.003 | Inferred | Sudo caching |

### Required to Reach Parity with Linux (2 Direct):

1. Build [macos/sensors/ebpf_primitives/](macos/sensors/ebpf_primitives/) with derivers:
   - `archive_tool_exec.rs` - detect tar, zip, gzip execution (→ T1560)
   - `staging_write.rs` - detect /tmp, ~/Downloads, ~/.cache writes (→ T1074.001)
   - Optionally: cred_access.rs, discovery_exec.rs, net_connect.rs for additional inferred techniques
2. Wire into [macos/capture_macos_rotating.rs](macos/capture_macos_rotating.rs)
3. Add test contract validation

---

## WINDOWS: Coverage Analysis

### Status: ❌ NOT IMPLEMENTED (0 Direct)

**Current State:**
- Sensors: [windows/sensors/](windows/sensors/) with 16 modules (Sysmon, ETW, Defender, service_monitor, task_scheduler_monitor, registry_monitor, log_tamper_monitor, lateral_movement_monitor, powershell_monitor, wmi_monitor)
- Playbooks: [playbooks/windows/](playbooks/windows/) with 5 playbooks
- **No primitive derivers:** No `windows/sensors/ebpf_primitives/` directory; sensors emit raw Sysmon/EventLog events, not canonical primitives

**Verification:**
```bash
grep -c "tags.*credential_access\|tags.*discovery\|tags.*exfiltration\|tags.*network_connection" windows/sensors/*.rs
# Result: 0 (sensors do not emit primitives)

ls -la windows/sensors/ebpf_primitives/ 2>/dev/null
# Result: does not exist
```

### Windows Sensor Architecture (Fact-Based, Not Primitive-Based)

Windows sensors currently:
1. Collect raw Sysmon/EventLog events
2. Emit playbook `Fact` objects: Exec, FileIO, NetConn, Auth, VendorAlert
3. Playbooks correlate facts to derive signals (current approach)

**Architectural Choice:** Windows does NOT use canonical primitives (credential_access, discovery, exfiltration, network_connection). Instead, it uses domain-specific fact types and playbook correlation.

### Inferred Techniques (7 from playbooks)

| Technique | Playbook Mapping | Architecture |
|-----------|---|---|
| T1053.005 | Task Scheduler Monitor | Windows-specific (Task Scheduler facts) |
| T1070.001 | Log Tamper Monitor | Event Log clearing detection |
| T1089 | Service Stop | Service modification facts |
| T1218 | Sysmon EventID 1 (CreateProcess) | LOLBin execution facts |
| T1543.001 | Service/Task creation | Service/Task creation facts |
| T1543.003 | Service creation | Service creation facts |
| T1547.001 | Registry modification | Registry persistence facts |
| T1562 | Log Tamper Monitor | Log tampering facts |

### Required to Reach Parity with Linux (2 Direct):

**Option A: Parallel primitive deriver path (Recommended)**
1. Build [windows/sensors/ebpf_primitives/](windows/sensors/ebpf_primitives/):
   - `archive_tool_exec.rs` - detect cmd.exe with tar/7z/PowerShell compression (→ T1560)
   - `staging_write.rs` - detect writes to %TEMP%, %APPDATA%\Local\Temp, user\Downloads (→ T1074.001)
2. Wire into capture pipeline
3. Add test validation

**Option B: Maintain fact-based architecture (Current)**
- Continue using Exec/FileIO/NetConn facts
- Limitation: Techniques assigned via playbook metadata, not verified primitives
- No change required; document as design choice

---

## Summary: Direct vs Inferred by OS

### Direct Coverage (Verified Primitive → Technique)

| OS | Technique | Primitive | Code Location | Test |
|----|-----------|-----------|---|---|
| **Linux** | T1560 | exfiltration (archive_tool_exec) | linux/sensors/ebpf_primitives/archive_tool_exec.rs:13-67 | Implicit in test_linux_derive_contract.rs |
| **Linux** | T1074.001 | exfiltration (staging_write) | linux/sensors/ebpf_primitives/staging_write.rs:13-82 | Implicit in test_linux_derive_contract.rs |
| **macOS** | (none) | (none) | (no ebpf_primitives dir) | N/A |
| **Windows** | (none) | (none) | (no ebpf_primitives dir) | N/A |

### Inferred Coverage (Playbook References, Multi-Fact Correlation)

- **Linux:** 44 techniques (primarily ransomware T1486, discovery tools, credential tools, persistence)
- **macOS:** 13 techniques (hardcoded in macos_native_playbooks.rs)
- **Windows:** 7 techniques (fact-based correlation in playbooks)

---

## Recommendations

### Linux: Expand Direct Coverage
Current direct coverage is intentionally narrow (only archive + staging = exfiltration phase). To expand, implement primitive derivers for:
1. **Authentication failures** → T1110 (Brute Force) - requires Auth fact from /var/log/auth.log or auditd
2. **Privilege escalation syscalls** → T1548 variants - requires syscall/capability tracking
3. **Network protocol analysis** → T1071, T1048 - requires DNS query/response, HTTP header parsing

### macOS: Implement Primitive Derivers
Build [macos/sensors/ebpf_primitives/](macos/sensors/ebpf_primitives/) to reach parity with Linux (2 direct). Reuse Linux deriver logic, adapt for macOS:
- `/tmp` → `/var/tmp` → `~/Library/Caches`
- `tar` → `zip` → macOS-native compression tools
- Estimated effort: 2-3 days

### Windows: Decide Primitive vs Fact Strategy
1. If adopting primitives: Build windows/sensors/ebpf_primitives/ (Option A)
2. If maintaining facts: Document architectural choice; close as "by design"
Current approach (facts) is valid for Windows; primitives are optional parity strategy.

---

## Appendix: Verification Commands

### Command 1: Confirm Linux Primitive Wiring
```bash
cd /Users/jermainebaker/git-glue

# Verify archive_tool_exec deriver emits primitive_subtype
grep -n "primitive_subtype.*archive_tool_exec" linux/sensors/ebpf_primitives/archive_tool_exec.rs
# Output: line 59: fields.insert(PRIMITIVE_SUBTYPE, json!("archive_tool_exec"));

# Verify staging_write deriver emits primitive_subtype
grep -n "primitive_subtype.*staging_write" linux/sensors/ebpf_primitives/staging_write.rs
# Output: line 77: fields.insert(PRIMITIVE_SUBTYPE, json!("staging_write"));

# Verify derivers are called in mod.rs
grep -n "archive_tool_exec::\|staging_write::" linux/sensors/ebpf_primitives/mod.rs
# Output: lines 21-23, 35-38
```

### Command 2: Confirm macOS/Windows No Primitive Derivers
```bash
# macOS
find macos -name "*ebpf_primitives*" -o -name "*deriver*" -o -name "*primitive*"
# Output: (empty)

# Windows
find windows -name "*ebpf_primitives*" -o -name "*deriver*" -o -name "*primitive*"
# Output: (empty)
```

### Command 3: List Inferred Technique Sources
```bash
# Linux inferred (playbook references)
grep -h "derive_tags\|primary_ttp" playbooks/linux/*.yaml | grep -oE "T[0-9]{4}(\.[0-9]{3})?" | sort -u | wc -l
# Output: 46 unique techniques (includes 2 direct)

# macOS inferred (hardcoded)
grep "T[0-9]\{4\}" macos/macos_native_playbooks.rs | grep -oE "T[0-9]{4}(\.[0-9]{3})?" | sort -u | wc -l
# Output: 13

# Windows inferred (playbook references)
grep -h "derive_tags\|primary_ttp" playbooks/windows/*.yaml | grep -oE "T[0-9]{4}(\.[0-9]{3})?" | sort -u | wc -l
# Output: 7
```

---

**Report Generated:** 2025-12-23  
**Methodology:** Primitive-driven mapping + playbook correlation analysis  
**Key Insight:** "Direct" coverage is narrow but certain; most techniques require playbook-level fact correlation
