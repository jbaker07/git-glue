# Windows Sensors Integration Guide

## Overview

The Windows sensor suite provides comprehensive telemetry collection across multiple Windows sources:
- **Event Log Sources**: Security, System, Application logs
- **Sysmon Integration**: Extended process/registry/network monitoring
- **ETW (Event Tracing for Windows)**: Kernel-level visibility (feature-gated)
- **Defender/AMSI**: Real-time threat detection
- **Specialized Monitors**: Task scheduling, services, registry, lateral movement

## Architecture

```
Windows Event Sources
    ├── Event Logs (EVT/EVTX)
    │   ├── evtx_collector.rs          → EVTX file parsing
    │   ├── service_monitor.rs         → Service lifecycle (7045, 4697)
    │   ├── task_scheduler_monitor.rs  → Task creation (106, 140, 200)
    │   ├── registry_monitor.rs        → Registry modifications (12-14)
    │   ├── log_tamper_monitor.rs      → Log tampering detection (1102, 4688)
    │   └── lateral_movement_monitor.rs → RDP/SMB/Kerberos (4624, 4769, 5140)
    │
    ├── Sysmon Integration
    │   ├── sysmon_adapter.rs          → Core events (1, 3, 5, 6, 7, 9, 11)
    │   └── sysmon_adapter_ext.rs      → Extended events (8, 10, 15, 17-21, 25, 26)
    │
    ├── ETW (Kernel-level)
    │   └── etw_adapter.rs             → Event Tracing for Windows (feature-gated)
    │
    ├── Real-time Protection
    │   └── defender_adapter.rs        → Windows Defender/AMSI logs
    │
    └── Data Pipeline
        └── capture_windows_rotating.rs → JSONL segments + index + heartbeat
```

## Sensor Modules

### 1. **Service Monitor** (`service_monitor.rs`)
Monitors Windows service creation and modification for persistence detection.

**Monitored Events:**
- `7045` (System log): Service installed
- `4697` (Security log): Service registry object modification

**Key Detections:**
- Services with suspicious binary paths
- Services created by non-admin accounts
- Service property modifications post-installation

**Playbook:** `signal_service_persistence.yaml`

Example event:
```rust
ServiceEvent {
    event_id: 7045,
    log_name: "System",
    ts: 2024-01-15T10:30:45Z,
    computer: "WORKSTATION-01",
    fields: {
        "ServiceName": "BackdoorService",
        "ImagePath": "C:\\Temp\\malware.exe",
        "StartType": "auto",
    }
}
```

### 2. **Task Scheduler Monitor** (`task_scheduler_monitor.rs`)
Detects scheduled task creation and modification for persistence.

**Monitored Events:**
- `106` (TaskScheduler/Operational): Task registered (created)
- `140`: Task updated
- `200`: Task executed

**Key Detections:**
- Tasks in unusual locations
- Non-standard task executables
- Tasks triggered on system start/logon
- Privilege escalation via task scheduling

**Playbook:** `signal_task_persistence.yaml`

Example event:
```rust
TaskSchedulerEvent {
    event_id: 106,
    ts: 2024-01-15T10:30:45Z,
    computer: "WORKSTATION-01",
    fields: {
        "TaskName": "WindowsUpdate",
        "TaskPath": "\\Microsoft\\Windows\\",
        "Author": "DOMAIN\\attacker",
    }
}
```

### 3. **Registry Monitor** (`registry_monitor.rs`)
Monitors registry key/value modifications (Sysmon EventID 12-14).

**Monitored Events:**
- `12` (Sysmon): Registry object created/deleted
- `13`: Registry value set
- `14`: Registry object renamed

**Key Detections:**
- Run/RunOnce key modifications
- IFEO debugger hijacking
- Services registry modifications
- Winlogon shell overrides

**Playbook:** `signal_registry_persistence.yaml`

Critical persistence locations:
- `HKLM\Software\Microsoft\Windows\CurrentVersion\Run*`
- `HKLM\Software\Microsoft\Windows NT\CurrentVersion\Winlogon`
- `HKLM\System\CurrentControlSet\Services\*`
- `HKLM\Software\Microsoft\Windows NT\CurrentVersion\Image File Execution Options`

### 4. **Log Tamper Monitor** (`log_tamper_monitor.rs`)
Detects event log tampering and evasion attempts.

**Monitored Events:**
- `4688` (Security): Process created with log tampering tools
- `1102` (Security): Security log cleared
- `104` (System): System log cleared

**Key Detections:**
- `wevtutil` execution with `clear` commands
- PowerShell `Clear-EventLog` / `Remove-EventLog`
- Direct log deletion attempts
- Suspicious process creation patterns

**Playbook:** `signal_log_tamper_detection.yaml`

⚠️ **Critical:** Log tampering is a strong indicator of active exploitation or post-compromise cleanup.

### 5. **Lateral Movement Monitor** (`lateral_movement_monitor.rs`)
Detects cross-system lateral movement patterns.

**Monitored Events:**
- `4624` (Security): Successful logon
- `4769`: Kerberos service ticket requested
- `5140`: Network share object accessed
- Sysmon `1`: Remote process creation (psexec, WMI)

**Key Detections:**
- Remote RDP logons (type 10) from external IPs
- Administrative share access (C$, IPC$, ADMIN$)
- Kerberos ticket patterns (pass-the-hash indicators)
- Remote code execution tools (psexesvc, WmiPrvSE)

**Playbook:** `signal_lateral_movement_detection.yaml`

## Configuration

### Enable/Disable Sensors

Edit `WindowsCapabilityConfig`:

```rust
pub struct WindowsCapabilityConfig {
    pub enable_sysmon: bool,
    pub enable_sysmon_ext: bool,
    pub enable_etw: bool,
    pub enable_defender: bool,
    pub enable_service_monitor: bool,
    pub enable_task_scheduler: bool,
    pub enable_registry_monitor: bool,
    pub enable_log_tamper_monitor: bool,
    pub enable_lateral_movement_monitor: bool,
    pub rate_limit: u32,
    pub redact_sensitive: bool,
}

// Example: Disable non-critical sensors for resource-constrained systems
let config = WindowsCapabilityConfig {
    enable_sysmon: true,
    enable_sysmon_ext: false,  // Reduce overhead
    enable_etw: false,
    enable_service_monitor: true,
    enable_task_scheduler: true,
    enable_registry_monitor: true,
    enable_log_tamper_monitor: true,
    enable_lateral_movement_monitor: true,
    rate_limit: 500,  // Lower rate limit
    redact_sensitive: true,
};
```

### Event Log Requirements

**Minimum Audit Policy:**

```powershell
# Enable Security event log
auditpol /set /category:"Logon/Logoff" /success:enable /failure:enable
auditpol /set /category:"Account Management" /success:enable /failure:enable
auditpol /set /category:"Object Access" /success:enable /failure:enable
auditpol /set /category:"Detailed Tracking" /success:enable /failure:enable

# Enable TaskScheduler operational log
Get-LogProperties "Microsoft-Windows-TaskScheduler/Operational" | Set-LogProperties -Enabled $true

# Enable Sysmon (requires separate installation)
# https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon
```

## Canonical Event Mapping

All Windows-specific events are converted to canonical `core::Event` format:

```rust
pub struct Event {
    pub ts: DateTime<Utc>,
    pub event_type: EventType,
    pub pid: Option<u32>,
    pub exe: Option<String>,
    pub tags: Vec<String>,
    // ... other fields
}
```

**Example conversion chain:**

```
Windows Event (7045, Service Installed)
    ↓
ServiceEvent struct
    ↓
ServiceMonitor::to_canonical_event()
    ↓
core::Event {
    event_type: EventType::Other,
    tags: ["windows_service", "service_install", "persistence", "BackdoorSvc"],
    ...
}
    ↓
Signal processing/correlation
    ↓
Playbook matching & response
```

## Playbook Integration

Each sensor has a corresponding detection playbook:

| Sensor | Playbook | Focus |
|--------|----------|-------|
| service_monitor | `signal_service_persistence.yaml` | Service-based persistence |
| task_scheduler_monitor | `signal_task_persistence.yaml` | Task scheduling persistence |
| registry_monitor | `signal_registry_persistence.yaml` | Registry-based persistence |
| log_tamper_monitor | `signal_log_tamper_detection.yaml` | Evasion/cleanup detection |
| lateral_movement_monitor | `signal_lateral_movement_detection.yaml` | Cross-system movement |

Each playbook includes:
- Signal matching criteria
- Correlation rules with time windows
- Response actions (block, isolate, alert)
- Remediation procedures
- Whitelisting/tuning options

## Performance & Tuning

### High-Volume Event Handling

Windows event logs can generate thousands of events/second. Tuning strategies:

```rust
// 1. Rate limiting (per-source)
config.rate_limit = 1000;  // events/sec cap

// 2. Filtering at source
// - Only collect Security log (ignore Application/System noise)
// - Subscribe to specific event IDs (4688, 4624, 7045, etc.)

// 3. Deduplication
// - Filter duplicate process creation events within 100ms
// - Collapse similar logon events from same user

// 4. Aggregation
// - Roll up high-volume events (e.g., failed logons)
// - Summary statistics (process frequency, unique user count)
```

### Sysmon vs ETW Tradeoff

- **Sysmon**: User-mode filtering, lower overhead, file-based
- **ETW**: Kernel-level, real-time, requires elevation

For resource-constrained systems, disable ETW and extended Sysmon:

```rust
WindowsCapabilityConfig {
    enable_sysmon: true,
    enable_sysmon_ext: false,  // Disable extended events (8, 10, 15, etc.)
    enable_etw: false,         // Disable kernel-level tracing
    rate_limit: 500,
    ..Default::default()
}
```

## Testing

Each sensor includes unit tests:

```bash
# Test service monitor
cargo test --lib windows::sensors::service_monitor

# Test task scheduler
cargo test --lib windows::sensors::task_scheduler_monitor

# Run all Windows sensor tests
cargo test --lib windows::sensors
```

Example test:

```rust
#[test]
fn test_service_install() {
    let event = ServiceEvent {
        event_id: 7045,
        log_name: "System".to_string(),
        ts: chrono::Utc::now(),
        fields: {
            let mut m = BTreeMap::new();
            m.insert("ServiceName".to_string(), "Backdoor".to_string());
            m.insert("ImagePath".to_string(), "C:\\Temp\\malware.exe".to_string());
            m
        },
    };

    let canonical = ServiceMonitor::to_canonical_event(&event);
    assert!(canonical.is_some());
    assert!(canonical.unwrap().tags.contains(&"service_install".to_string()));
}
```

## Troubleshooting

### Missing Event Logs

**Problem:** No events collected from specific log.

**Solutions:**
1. Verify log is enabled: `Get-LogProperties -Name "Security" | Format-Table`
2. Check log size: `Get-LogProperties -Name "Security" | Select MaximumSizeInBytes`
3. Enable if disabled: `Set-LogProperties -Name "Security" -Enabled $true`
4. Check permissions: Event logs require SYSTEM/Administrator access

### High CPU/Memory Usage

**Problem:** Sensor consuming excessive resources.

**Solutions:**
1. Lower `rate_limit` in config
2. Disable extended Sysmon events (`enable_sysmon_ext: false`)
3. Disable ETW (`enable_etw: false`)
4. Increase segment rotation size (batch writes)
5. Filter at source (specific event IDs only)

### Missing Sysmon Events

**Problem:** No Sysmon EventID 12-14 (registry events).

**Solutions:**
1. Install Sysmon: https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon
2. Verify config includes registry filtering
3. Check Sysmon is running: `Get-Process sysmon*`
4. Review Sysmon event log: `Get-WinEvent -LogName "Microsoft-Windows-Sysmon/Operational"`

## References

- **Event IDs**: https://docs.microsoft.com/en-us/windows/security/threat-protection/auditing/audit-events
- **Sysmon**: https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon
- **MITRE ATT&CK**: https://attack.mitre.org/tactics/enterprise/
- **Windows Event Log Cheatsheet**: https://www.ultimatewindowssecurity.com/securitylog/book/

## Maintenance

### Regular Updates

Windows sensor configuration should be reviewed quarterly:
- New persistence techniques (update playbooks)
- Event log audit policy changes
- Sysmon configuration updates
- Performance tuning based on operational metrics

### Retention Policy

Set appropriate event log retention:

```powershell
# Security log: 90 days
wevtutil.exe set-log Security /retention:true /maxsize:1073741824

# System log: 30 days
wevtutil.exe set-log System /retention:true /maxsize:536870912

# TaskScheduler: 30 days
wevtutil.exe set-log "Microsoft-Windows-TaskScheduler/Operational" /retention:true /maxsize:536870912
```
