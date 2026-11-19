# Windows Sensors Suite - Complete Documentation Index

## 📚 Documentation

### Executive & Planning
- **[WINDOWS_SENSORS_EXECUTIVE_SUMMARY.md](WINDOWS_SENSORS_EXECUTIVE_SUMMARY.md)** (389 lines)
  - High-level overview for decision makers
  - Business impact and ROI
  - Success criteria and risk assessment
  - Deployment recommendations

- **[WINDOWS_SENSORS_COMPLETION.md](WINDOWS_SENSORS_COMPLETION.md)** (198 lines)
  - Implementation status for all 5 sensors
  - Build status and quality metrics
  - File manifest with line counts
  - Future enhancement roadmap

- **[WINDOWS_SENSORS_DEPLOYMENT_CHECKLIST.md](WINDOWS_SENSORS_DEPLOYMENT_CHECKLIST.md)** (356 lines)
  - Pre-deployment validation
  - Step-by-step deployment procedures
  - Post-deployment testing
  - Rollback procedures
  - Success criteria checklist

### Technical & Operations
- **[WINDOWS_SENSORS_INTEGRATION.md](WINDOWS_SENSORS_INTEGRATION.md)** (448 lines)
  - Complete architecture overview
  - Detailed sensor module documentation
  - Configuration guide with examples
  - Performance tuning strategies
  - Troubleshooting procedures
  - Testing and validation methods

- **[WINDOWS_SENSORS_QUICK_REFERENCE.md](WINDOWS_SENSORS_QUICK_REFERENCE.md)** (341 lines)
  - Quick start guide
  - Event ID reference table
  - Common detection patterns
  - Response action commands
  - Configuration snippets
  - Testing commands

---

## 🔍 Sensor Modules

### 1. Service Monitor
**File**: `windows/sensors/service_monitor.rs` (104 lines)

| Aspect | Details |
|--------|---------|
| **Events** | 7045, 4697 |
| **Detection** | Service installation, registry modification |
| **Severity** | HIGH/CRITICAL |
| **Playbook** | signal_service_persistence.yaml |
| **Test Coverage** | ✅ Basic service install |

**Quick Start**:
```rust
let event = ServiceEvent {
    event_id: 7045,
    log_name: "System".to_string(),
    fields: { /* ... */ }
};
let canonical = ServiceMonitor::to_canonical_event(&event);
```

### 2. Task Scheduler Monitor
**File**: `windows/sensors/task_scheduler_monitor.rs` (108 lines)

| Aspect | Details |
|--------|---------|
| **Events** | 106, 140, 200 |
| **Detection** | Task creation, modification, execution |
| **Severity** | HIGH/MEDIUM |
| **Playbook** | signal_task_persistence.yaml |
| **Test Coverage** | ✅ Task creation & execution |

**Quick Start**:
```rust
let event = TaskSchedulerEvent {
    event_id: 106,
    fields: { /* ... */ }
};
let canonical = TaskSchedulerMonitor::to_canonical_event(&event);
```

### 3. Registry Monitor
**File**: `windows/sensors/registry_monitor.rs` (89 lines)

| Aspect | Details |
|--------|---------|
| **Events** | Sysmon 12, 13, 14 |
| **Detection** | Registry key/value modifications |
| **Severity** | HIGH/CRITICAL |
| **Playbook** | signal_registry_persistence.yaml |
| **Test Coverage** | ✅ Persistence key detection |

**Quick Start**:
```rust
let event = RegistryEvent {
    event_id: 13,
    fields: { /* key and value data */ }
};
let canonical = RegistryMonitor::to_canonical_event(&event);
```

### 4. Log Tamper Monitor
**File**: `windows/sensors/log_tamper_monitor.rs` (104 lines)

| Aspect | Details |
|--------|---------|
| **Events** | 4688, 1102, 104 |
| **Detection** | Log clearing, wevtutil/PowerShell patterns |
| **Severity** | **CRITICAL** |
| **Playbook** | signal_log_tamper_detection.yaml |
| **Test Coverage** | ✅ wevtutil detection, log clearing |

**Quick Start**:
```rust
let event = LogTamperEvent {
    event_id: 4688,
    fields: { "CommandLine": "wevtutil cl Security".into() }
};
let canonical = LogTamperMonitor::to_canonical_event(&event);
```

### 5. Lateral Movement Monitor
**File**: `windows/sensors/lateral_movement_monitor.rs` (149 lines)

| Aspect | Details |
|--------|---------|
| **Events** | 4624, 4769, 5140, Sysmon 1 |
| **Detection** | RDP, SMB, Kerberos, remote execution |
| **Severity** | CRITICAL/HIGH |
| **Playbook** | signal_lateral_movement_detection.yaml |
| **Test Coverage** | ✅ RDP, share access, Kerberos |

**Quick Start**:
```rust
let event = LateralMovementEvent {
    event_id: 4624,
    fields: { "LogonType": "10".into(), /* ... */ }
};
let canonical = LateralMovementMonitor::to_canonical_event(&event);
```

---

## 📋 Playbooks

### Service Persistence
**File**: `playbooks/windows/signal_service_persistence.yaml` (79 lines)

```yaml
name: "Windows Service Installation"
signals:
  - tag: "windows_service"
  - tag: "service_install"
severity: HIGH/CRITICAL
actions:
  - isolate_service
  - quarantine_binary
  - alert
```

**Trigger**: Service installed with suspicious binary path  
**Response**: Service stop, binary quarantine  
**Evidence**: Service name, path, start type, account

### Task Persistence
**File**: `playbooks/windows/signal_task_persistence.yaml` (85 lines)

```yaml
name: "Windows Task Scheduler Persistence"
signals:
  - tag: "windows_scheduler"
  - tag: "task_create"
severity: HIGH/MEDIUM
actions:
  - suspend_task
  - block_author
  - alert
```

**Trigger**: Task created with non-standard path  
**Response**: Task suspension, author investigation  
**Evidence**: Task name, path, author, executable

### Registry Persistence
**File**: `playbooks/windows/signal_registry_persistence.yaml` (124 lines)

```yaml
name: "Windows Registry Persistence"
signals:
  - tag: "windows_registry"
  - tag: "registry_value_set"
severity: HIGH/CRITICAL
actions:
  - snapshot_registry
  - revert_registry
  - alert
```

**Trigger**: Registry value set in Run/IFEO/Winlogon key  
**Response**: Registry value removal, system scan  
**Evidence**: Registry path, value, modifying process

### Log Tamper Detection
**File**: `playbooks/windows/signal_log_tamper_detection.yaml` (118 lines)

```yaml
name: "Windows Log Tampering Detection"
signals:
  - tag: "windows_log_tamper"
  - tag: "evasion"
severity: CRITICAL
actions:
  - freeze_evidence
  - isolate_host
  - preserve_memory
  - alert
```

**Trigger**: Event log cleared OR log tool executed  
**Response**: IMMEDIATE isolation, memory dump, forensics  
**Evidence**: Log clear timestamp, user, process chain

### Lateral Movement Detection
**File**: `playbooks/windows/signal_lateral_movement_detection.yaml` (147 lines)

```yaml
name: "Windows Lateral Movement Detection"
signals:
  - tag: "windows_lateral_movement"
  - tag: "remote_logon"
severity: CRITICAL/HIGH
actions:
  - terminate_session
  - block_ip
  - reset_credentials
  - alert
```

**Trigger**: RDP/SMB access from external IP  
**Response**: Session termination, IP blocking, credential reset  
**Evidence**: Source IP, target user, logon type, process

---

## 🛠️ Configuration

### Full Configuration Example

```rust
pub struct WindowsCapabilityConfig {
    // Core sensors
    pub enable_sysmon: bool,           // Process & registry events
    pub enable_sysmon_ext: bool,       // Extended Sysmon (8, 10, 15, etc.)
    pub enable_etw: bool,              // Kernel-level tracing
    pub enable_defender: bool,         // Real-time protection logs
    
    // Specialized monitors
    pub enable_service_monitor: bool,              // Service persistence
    pub enable_task_scheduler: bool,               // Task persistence
    pub enable_registry_monitor: bool,             // Registry persistence
    pub enable_log_tamper_monitor: bool,           // Log tampering
    pub enable_lateral_movement_monitor: bool,     // Lateral movement
    
    // Performance & security
    pub rate_limit: u32,               // events/sec cap
    pub redact_sensitive: bool,        // Hide PowerShell commands, etc.
}

// Default configuration
impl Default for WindowsCapabilityConfig {
    fn default() -> Self {
        Self {
            enable_sysmon: true,
            enable_sysmon_ext: true,
            enable_etw: cfg!(feature = "windows_etw"),
            enable_defender: true,
            enable_service_monitor: true,
            enable_task_scheduler: true,
            enable_registry_monitor: true,
            enable_log_tamper_monitor: true,
            enable_lateral_movement_monitor: true,
            rate_limit: 1000,
            redact_sensitive: true,
        }
    }
}
```

### Environment-Specific Configs

**High-Security (Corporate Network)**
```rust
let config = WindowsCapabilityConfig::default();  // All enabled
```

**Resource-Constrained (Edge Devices)**
```rust
let config = WindowsCapabilityConfig {
    enable_sysmon_ext: false,  // Disable extended events
    enable_etw: false,         // Disable kernel-level
    rate_limit: 500,           // Lower rate limit
    ..Default::default()
};
```

**Performance-Optimized (High-Volume)**
```rust
let config = WindowsCapabilityConfig {
    enable_sysmon: true,
    enable_sysmon_ext: false,
    enable_etw: false,
    enable_defender: false,
    rate_limit: 2000,          // Higher rate limit
    ..Default::default()
};
```

---

## 🧪 Testing

### Run All Tests
```bash
cargo test --lib windows::sensors
```

### Test Specific Sensors
```bash
# Service monitor
cargo test --lib windows::sensors::service_monitor

# Task scheduler
cargo test --lib windows::sensors::task_scheduler_monitor

# Registry monitor
cargo test --lib windows::sensors::registry_monitor

# Log tamper
cargo test --lib windows::sensors::log_tamper_monitor

# Lateral movement
cargo test --lib windows::sensors::lateral_movement_monitor
```

### Example Test
```rust
#[test]
fn test_service_install() {
    let event = ServiceEvent {
        event_id: 7045,
        log_name: "System".to_string(),
        ts: chrono::Utc::now(),
        fields: {
            let mut m = BTreeMap::new();
            m.insert("ServiceName".to_string(), "BackdoorSvc".to_string());
            m.insert("ImagePath".to_string(), "C:\\Temp\\malware.exe".to_string());
            m
        },
    };

    let canonical = ServiceMonitor::to_canonical_event(&event);
    assert!(canonical.is_some());
    let evt = canonical.unwrap();
    assert!(evt.tags.iter().any(|t| t == "service_install"));
}
```

---

## 📊 Event Coverage

### Event ID Reference

| Event ID | Log | Sensor | Detection |
|----------|-----|--------|-----------|
| **7045** | System | Service | Service installed |
| **4697** | Security | Service | Service registry modified |
| **106** | TaskScheduler | Task | Task registered |
| **140** | TaskScheduler | Task | Task updated |
| **200** | TaskScheduler | Task | Task executed |
| **12** | Sysmon | Registry | Registry object created/deleted |
| **13** | Sysmon | Registry | Registry value set |
| **14** | Sysmon | Registry | Registry object renamed |
| **4688** | Security | LogTamper | Process created (detect tools) |
| **1102** | Security | LogTamper | Security log cleared |
| **104** | System | LogTamper | System log cleared |
| **4624** | Security | LateralMovement | Successful logon |
| **4769** | Security | LateralMovement | Kerberos ticket requested |
| **5140** | Security | LateralMovement | Network share accessed |

### MITRE ATT&CK Coverage

| Technique | ID | Sensor | Playbook |
|-----------|----|----|----------|
| Service Installation | T1543 | Service | signal_service_persistence |
| Scheduled Task | T1053 | Task | signal_task_persistence |
| Registry Modification | T1112 | Registry | signal_registry_persistence |
| Clear Event Logs | T1070.001 | LogTamper | signal_log_tamper_detection |
| Remote Logon | T1021.001 | LateralMovement | signal_lateral_movement_detection |
| SMB Shares | T1021.002 | LateralMovement | signal_lateral_movement_detection |

---

## 📁 File Structure

```
git-glue/
├── windows/sensors/
│   ├── service_monitor.rs                    ✅ 104 lines
│   ├── task_scheduler_monitor.rs             ✅ 108 lines
│   ├── registry_monitor.rs                   ✅ 89 lines
│   ├── log_tamper_monitor.rs                 ✅ 104 lines
│   ├── lateral_movement_monitor.rs           ✅ 149 lines
│   ├── mod.rs                                ✅ Updated
│   ├── sysmon_adapter_ext.rs                 ✅ 221 lines
│   ├── evtx_collector.rs                     (Pre-existing)
│   ├── etw_adapter.rs                        (Pre-existing)
│   ├── defender_adapter.rs                   (Pre-existing)
│   └── capture_windows_rotating.rs           (Pre-existing)
│
├── playbooks/windows/
│   ├── signal_service_persistence.yaml       ✅ 79 lines
│   ├── signal_task_persistence.yaml          ✅ 85 lines
│   ├── signal_registry_persistence.yaml      ✅ 124 lines
│   ├── signal_log_tamper_detection.yaml      ✅ 118 lines
│   ├── signal_lateral_movement_detection.yaml ✅ 147 lines
│   └── signal_*.yaml                         (Pre-existing, 3)
│
└── docs/
    ├── WINDOWS_SENSORS_EXECUTIVE_SUMMARY.md  ✅ 389 lines
    ├── WINDOWS_SENSORS_COMPLETION.md         ✅ 198 lines
    ├── WINDOWS_SENSORS_INTEGRATION.md        ✅ 448 lines
    ├── WINDOWS_SENSORS_QUICK_REFERENCE.md    ✅ 341 lines
    ├── WINDOWS_SENSORS_DEPLOYMENT_CHECKLIST.md ✅ 356 lines
    └── WINDOWS_SENSORS_DOCUMENTATION_INDEX.md ✅ This file
```

---

## 🚀 Quick Start

### 1. Enable Sensors in Configuration
```rust
let config = WindowsCapabilityConfig::default();
```

### 2. Verify Event Logs Enabled
```powershell
Get-LogProperties -Name Security | Select Enabled
Get-LogProperties -Name System | Select Enabled
```

### 3. Build & Test
```bash
cargo build --release
cargo test --lib windows::sensors --release
```

### 4. Deploy
See `WINDOWS_SENSORS_DEPLOYMENT_CHECKLIST.md` for full procedures

### 5. Monitor
Check sensor health via dashboard or logs

---

## 📞 Support Matrix

| Need | Document | Contact |
|------|----------|---------|
| **Overview** | WINDOWS_SENSORS_EXECUTIVE_SUMMARY.md | Leadership |
| **Technical Details** | WINDOWS_SENSORS_INTEGRATION.md | Engineers |
| **Quick Reference** | WINDOWS_SENSORS_QUICK_REFERENCE.md | Operators |
| **Deployment** | WINDOWS_SENSORS_DEPLOYMENT_CHECKLIST.md | DevOps/SRE |
| **Implementation Status** | WINDOWS_SENSORS_COMPLETION.md | Project Managers |

---

## ✅ Verification Checklist

Before deploying, verify:

- [ ] All 5 sensor modules compile without errors
- [ ] All unit tests pass
- [ ] Event logs enabled on test systems
- [ ] Sysmon installed (if using registry monitoring)
- [ ] Appropriate audit policy configured
- [ ] Documentation reviewed by team
- [ ] Playbooks validated in staging
- [ ] Performance baselines established

---

## 📈 Metrics & Monitoring

### Key Performance Indicators

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Event Collection Latency** | < 5 sec | Time from event to processed |
| **Detection Accuracy** | > 90% | True positive rate |
| **False Positive Rate** | < 5% | False alerts per 1000 events |
| **CPU Overhead** | < 5% | Baseline increase |
| **Memory Footprint** | < 200 MB | RAM consumption |
| **Event Loss** | 0% | Dropped events |

### Monitoring Queries

**Verify event collection:**
```powershell
Get-WinEvent -FilterHashtable @{LogName='System'; ID=7045} | Measure-Object
Get-WinEvent -FilterHashtable @{LogName='Security'; ID=4624} | Measure-Object
```

**Check sensor health:**
```bash
# Monitor sensor logs
tail -f sensor_output.log

# Check memory usage
ps aux | grep sensor
```

---

## 🔐 Security Considerations

### Data Handling
- Sensitive data (PowerShell commands) redactable via config
- Event logs stored with appropriate access controls
- Evidence preservation maintains chain of custody

### Alert Handling
- Critical alerts (log tampering) immediate response
- Alert severity determines response automation level
- All actions logged for audit trail

### Compliance
- Event-based audit trail for regulatory requirements
- Structured data for compliance reporting
- Retention policies configurable

---

## 📚 Related Documentation

- [WINDOWS_SENSORS_EXECUTIVE_SUMMARY.md](WINDOWS_SENSORS_EXECUTIVE_SUMMARY.md) - Business overview
- [WINDOWS_SENSORS_INTEGRATION.md](WINDOWS_SENSORS_INTEGRATION.md) - Technical architecture
- [WINDOWS_SENSORS_QUICK_REFERENCE.md](WINDOWS_SENSORS_QUICK_REFERENCE.md) - Operational guide
- [WINDOWS_SENSORS_DEPLOYMENT_CHECKLIST.md](WINDOWS_SENSORS_DEPLOYMENT_CHECKLIST.md) - Deployment procedures
- [WINDOWS_SENSORS_COMPLETION.md](WINDOWS_SENSORS_COMPLETION.md) - Implementation status

---

## Version Information

| Component | Version | Status |
|-----------|---------|--------|
| Windows Sensors Suite | 1.0 | ✅ Complete |
| Documentation | 1.0 | ✅ Complete |
| Playbooks | 1.0 | ✅ Complete |
| Tests | 1.0 | ✅ Complete |

---

**Last Updated**: 2024-01-15  
**Status**: PRODUCTION READY  
**Maintainer**: git-glue Windows Security Team
