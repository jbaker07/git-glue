# Windows Sensors Quick Reference

## Sensor Overview

| Sensor | Events | Coverage | Severity |
|--------|--------|----------|----------|
| **Service Monitor** | 7045, 4697 | Service persistence | HIGH |
| **Task Scheduler** | 106, 140, 200 | Task persistence | HIGH |
| **Registry Monitor** | 12, 13, 14 (Sysmon) | Registry persistence | HIGH |
| **Log Tamper** | 4688, 1102, 104 | Evasion detection | **CRITICAL** |
| **Lateral Movement** | 4624, 4769, 5140, Sysmon 1 | Cross-system movement | CRITICAL |

---

## Quick Start

### Enable All Sensors
```rust
let config = WindowsCapabilityConfig::default();  // All enabled
```

### Disable High-Overhead Sensors
```rust
let config = WindowsCapabilityConfig {
    enable_sysmon_ext: false,  // Skip extended Sysmon
    enable_etw: false,         // Skip kernel-level tracing
    ..Default::default()
};
```

### Tune for Performance
```rust
let config = WindowsCapabilityConfig {
    rate_limit: 500,           // Reduce event rate
    enable_sysmon_ext: false,
    enable_etw: false,
    ..Default::default()
};
```

---

## Critical Event IDs to Monitor

### Service Persistence
```
7045   → Service installed
4697   → Service registry modified
```

### Task Persistence
```
106    → Task registered (created)
140    → Task updated
200    → Task executed
```

### Registry Persistence
```
12     → Registry object created/deleted
13     → Registry value set
14     → Registry object renamed
```

### Evasion Detection
```
1102   → Security log cleared [CRITICAL]
104    → System log cleared [CRITICAL]
4688   → Process created (detect log tools)
```

### Lateral Movement
```
4624   → Successful logon
4769   → Kerberos ticket requested
5140   → Network share accessed
1      → Process created (Sysmon)
```

---

## Common Detection Patterns

### 🔴 Service Backdoor (CRITICAL)
```
Event 7045: ServiceName="BadService" + ImagePath="C:\Temp\malware.exe"
→ Playbook: signal_service_persistence.yaml
→ Action: Stop service, quarantine binary
```

### 🔴 Task Persistence (HIGH)
```
Event 106: TaskPath="\\System32\\tasks\\..." + Author="DOMAIN\attacker"
→ Playbook: signal_task_persistence.yaml
→ Action: Suspend task, block author account
```

### 🔴 Registry Run Key (HIGH)
```
Event 13: TargetObject="...Run\Backdoor" + Details="C:\Temp\malware.exe"
→ Playbook: signal_registry_persistence.yaml
→ Action: Remove registry value, scan system
```

### 🟥 Log Tampering (CRITICAL)
```
Event 1102 or 104: [LOG CLEARED]
→ Playbook: signal_log_tamper_detection.yaml
→ Action: Isolate host immediately, memory dump
→ Priority: RESPOND WITHIN 5 MINUTES
```

### 🔴 Remote Access (HIGH)
```
Event 4624: LogonType=10 + SourceIP="external"
→ Playbook: signal_lateral_movement_detection.yaml
→ Action: Disconnect RDP session, block IP
```

---

## Playbook Matching

### Service Persistence
- **Trigger**: Tag="service_install" + Tag="persistence"
- **Conditions**: Binary path in %TEMP%, %APPDATA%, or removable media
- **Response**: Service stop, binary quarantine
- **File**: `playbooks/windows/signal_service_persistence.yaml`

### Task Persistence
- **Trigger**: Tag="task_create" + Tag="persistence"
- **Conditions**: Unusual task path, non-admin author
- **Response**: Task suspension, author investigation
- **File**: `playbooks/windows/signal_task_persistence.yaml`

### Registry Persistence
- **Trigger**: Tag="registry_value_set" + registry path matches Run/IFEO/Winlogon
- **Conditions**: Non-system process modifying persistence keys
- **Response**: Registry value removal, system scan
- **File**: `playbooks/windows/signal_registry_persistence.yaml`

### Log Tampering
- **Trigger**: Event 1102/104 (log cleared) OR Event 4688 + log tool detection
- **Conditions**: Any occurrence (NO WHITELIST)
- **Response**: IMMEDIATE isolation, memory dump, forensics
- **File**: `playbooks/windows/signal_log_tamper_detection.yaml`

### Lateral Movement
- **Trigger**: Remote logon (4624 type 3/10) + external IP OR admin share access (5140)
- **Conditions**: Non-admin user accessing shares, Kerberos patterns
- **Response**: Session termination, IP block, credential reset
- **File**: `playbooks/windows/signal_lateral_movement_detection.yaml`

---

## Event Log Setup

### Minimum Audit Policy
```powershell
# Security log - required
auditpol /set /category:"Logon/Logoff" /success:enable /failure:enable
auditpol /set /category:"Detailed Tracking" /success:enable /failure:enable
auditpol /set /category:"Object Access" /success:enable /failure:enable

# System log - optional (for service events)
# Enabled by default

# TaskScheduler - recommended
Get-LogProperties "Microsoft-Windows-TaskScheduler/Operational" | `
  Set-LogProperties -Enabled $true
```

### Verify Logs Enabled
```powershell
Get-LogProperties -Name Security
Get-LogProperties -Name System
Get-LogProperties -Name "Microsoft-Windows-TaskScheduler/Operational"
```

### Set Retention (90 days)
```powershell
wevtutil.exe set-log Security /retention:true /maxsize:1073741824
```

---

## Testing Sensors

### Run All Tests
```bash
cargo test --lib windows::sensors
```

### Test Specific Sensor
```bash
cargo test --lib windows::sensors::service_monitor
cargo test --lib windows::sensors::task_scheduler_monitor
cargo test --lib windows::sensors::registry_monitor
cargo test --lib windows::sensors::log_tamper_monitor
cargo test --lib windows::sensors::lateral_movement_monitor
```

---

## Troubleshooting

### No Events Collected
```powershell
# Check if logs are enabled
Get-LogProperties -Name Security | Select Enabled

# Check log size
Get-LogProperties -Name Security | Select MaximumSizeInBytes

# Enable if needed
Set-LogProperties -Name Security -Enabled $true
```

### Missing Sysmon Events
```powershell
# Verify Sysmon running
Get-Process sysmon*

# Check Sysmon event log
Get-WinEvent -LogName "Microsoft-Windows-Sysmon/Operational" | `
  Where-Object { $_.Id -eq 12 } | Select -First 5
```

### High CPU Usage
1. Lower `rate_limit` in config
2. Disable `enable_sysmon_ext`
3. Disable `enable_etw`
4. Filter to specific event IDs only

---

## Configuration Reference

```rust
WindowsCapabilityConfig {
    // Enable core sensors
    enable_sysmon: true,               // Required for registry/process events
    enable_sysmon_ext: true,           // Extended events (8, 10, 15, 17-21, 25, 26)
    enable_etw: cfg!(feature = "windows_etw"),  // Kernel-level (overhead!)
    enable_defender: true,             // Real-time protection logs
    
    // Enable specialized monitors
    enable_service_monitor: true,      // Service persistence detection
    enable_task_scheduler: true,       // Task scheduling detection
    enable_registry_monitor: true,     // Registry modification detection
    enable_log_tamper_monitor: true,   // Log tampering detection
    enable_lateral_movement_monitor: true,  // RDP/SMB/Kerberos detection
    
    // Performance tuning
    rate_limit: 1000,                  // events/sec cap
    
    // Security
    redact_sensitive: true,            // Redact PowerShell commands, etc.
}
```

---

## Response Actions

### Service Persistence Detected
```powershell
# 1. Stop service
sc stop ServiceName

# 2. Delete service
sc delete ServiceName

# 3. Quarantine executable
Move-Item C:\Path\to\malware.exe C:\Quarantine\
```

### Task Persistence Detected
```powershell
# 1. Suspend task
schtasks /change /tn "\Path\TaskName" /disable

# 2. Delete task
schtasks /delete /tn "\Path\TaskName" /f

# 3. Investigate parent process
Get-Process | Where-Object { $_.Handle -eq <pid> }
```

### Registry Persistence Detected
```powershell
# 1. Backup registry
reg export HKLM\path\to\key output.reg

# 2. Remove malicious value
reg delete HKLM\path\to\key /v ValueName /f

# 3. Verify removal
reg query HKLM\path\to\key /v ValueName
```

### Log Tampering Detected
```powershell
# 1. Export logs immediately
Get-WinEvent -ListLog * | Export-EventLog

# 2. Check for backups
dir $env:SYSTEMROOT\System32\winevt\Logs\

# 3. Isolate host
netsh advfirewall set allprofiles state on
```

### RDP Lateral Movement Detected
```powershell
# 1. Disconnect session
Disconnect-PSSession

# 2. Block IP at firewall
netsh advfirewall firewall add rule name="Block Attacker" dir=in action=block remoteip=<IP>

# 3. Force password reset
$user = Get-ADUser <username>
Set-ADAccountPassword -Identity $user -NewPassword (ConvertTo-SecureString -AsPlainText -Force "NewPassword123!")
```

---

## Severity Levels

| Level | Color | Response Time | Examples |
|-------|-------|----------------|----------|
| CRITICAL | 🔴 Red | Immediate (5 min) | Log tampering, code execution, active breach |
| HIGH | 🟠 Orange | Urgent (30 min) | Service backdoors, task persistence, RDP access |
| MEDIUM | 🟡 Yellow | Normal (4 hours) | Registry modifications, failed access attempts |
| LOW | 🟢 Green | Routine | Informational, baseline monitoring |

---

## Resources

- **Windows Event IDs**: https://docs.microsoft.com/en-us/windows/security/threat-protection/auditing/audit-events
- **Sysmon**: https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon
- **MITRE ATT&CK**: https://attack.mitre.org/tactics/enterprise/
- **Windows Forensics**: https://www.ired.team/

---

## Status

✅ All 5 sensors implemented and tested  
✅ All 5 playbooks created with response actions  
✅ Complete documentation provided  
✅ No compilation errors  
✅ Ready for production deployment
