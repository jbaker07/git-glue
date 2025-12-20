MODEL A PRODUCTION DEPLOYMENT GUIDE
===================================

## CRITICAL: Audit Subsystem Requirements

macOS 14+ disables the OpenBSM audit subsystem by default. Model A requires
an active audit stream to function. This section outlines enablement procedures.

### Audit Status Check

Before deploying Model A, verify audit is operational:

  $ sudo ./target/debug/capture_macos_rotating --telemetry-root /tmp/test

If audit is disabled, you will see:

  AUDIT SUBSYSTEM NOT OPERATIONAL
  [Detailed remediation steps provided by capture binary]

### Enabling Audit (One-Time Setup)

Follow these exact steps on a production system:

#### Step 1: Enable audit configuration (if not already present)

  $ sudo cp /etc/security/audit_control.example /etc/security/audit_control

#### Step 2: Optionally configure audit flags for visibility

Edit /etc/security/audit_control to enable process exec + args logging:

  $ sudo vim /etc/security/audit_control

Recommended settings for Model A:

  # Near the top, modify the "flags" line to include:
  # flags: lo,ad,fd,fm,-all,ex,pc  # ex=exec, pc=process class
  # policy: argv,arge  # argv=args, arge=args+env

After editing, enable:

  $ sudo launchctl enable system/com.apple.auditd
  $ sudo launchctl kickstart -k system/com.apple.auditd

#### Step 3: Reboot (required)

Apple's auditd service requires a full system reboot to activate:

  $ sudo shutdown -r now

#### Step 4: Verify audit is running

After reboot, verify with:

  $ launchctl list | grep auditd
  # Should show: com.apple.auditd (no -X code)

  $ praudit -l /dev/auditpipe | head -5
  # Should show live audit events

### Audit Visibility Guidance

OpenBSM event types exposed by /dev/auditpipe depend on audit flags in
/etc/security/audit_control. For Model A playbooks to work:

  **Minimum required:**
    - AUE_EXECVE (process execution with args)
    - AUE_OPEN_W / AUE_CREATE / AUE_UNLINK (file ops)
    - AUE_CONNECT (network connections)
    - AUE_SUDO (privilege transitions)

  **Flags to enable these:**
    flags: lo,ad,fd,fm,-all,ex,pc

  **Policy flags to enable args logging:**
    policy: argv,arge

### macOS Deprecation Risk & EndpointSecurity Fallback

The OpenBSM audit subsystem is deprecated and will be removed in a future
macOS version (timeline TBD by Apple). When audit is unavailable, Model A
will require pivoting to Apple's EndpointSecurity framework.

**Strategic Path (not implemented yet):**
  1. Current: OpenBSM audit (macOS 11-14+)
  2. Future: EndpointSecurity framework as primary sensor
  3. Hybrid: Both sensors active during transition period

---

## Deployment via LaunchD

### Build Release Binaries

  $ cargo build --release

### Install to System

  $ sudo -E ./scripts/macos/install_launchd.sh --telemetry-root /opt/modela/telemetry

This will:
  - Copy binaries to /opt/modela/bin
  - Copy playbooks to /opt/modela/playbooks/macos
  - Install LaunchDaemon for capture (root)
  - Install LaunchAgents for locald + ui_server (user)
  - Create /var/log/modela for logs

### Verify Installation

  $ launchctl list | grep modela
  # Should show: com.modela.capture, com.modela.locald, com.modela.ui_server

  $ tail -20 /var/log/modela/*.log

  $ curl http://127.0.0.1:8000/api/health | jq

### Uninstall

  $ sudo ./scripts/macos/uninstall_launchd.sh

---

## Playbooks (Production Starter Set)

Four production-ready playbooks are included:

### 1. user_writable_exec_egress.yaml
  **Detects:** Process execution from user-writable directories (/tmp, ~/Downloads, etc.)
              followed by outbound network connection
  **MITRE:** T1204 (User Execution), T1071 (Application Layer Protocol)
  **Severity:** HIGH
  **TTL:** 5 minutes (correlation window)

### 2. launch_agent_persistence.yaml
  **Detects:** LaunchAgent/Daemon plist writes + launchctl load/bootstrap
  **MITRE:** T1543.001 (Create/Modify System Process), T1547.011 (Boot Initialization)
  **Severity:** CRITICAL
  **TTL:** 10 minutes

### 3. osascript_suspicious.yaml
  **Detects:** osascript/AppleScript execution with suspicious arguments
              (curl, wget, sh, bash, python payloads)
  **MITRE:** T1059.002 (Command and Scripting Interpreter)
  **Severity:** HIGH
  **TTL:** 2 minutes

### 4. sudo_anomaly.yaml
  **Detects:** Unexpected sudo usage patterns (-u, -i, -s flags)
  **MITRE:** T1548.003 (Abuse Elevation Control Mechanism)
  **Severity:** MEDIUM
  **TTL:** 5 minutes

---

## Operational Safeguards

### Logging

Structured logs to /var/log/modela/:
  - capture.out.log: segment writes, rotation events
  - locald.out.log: fact extraction, incident fires
  - ui_server.out.log: API queries, deref operations

### Disk Usage

Segments rotate hourly. Default retention:
  - Max 100 segments retained (configurable via policy)
  - Each segment ~10-100MB (varies with audit traffic)
  - Total disk footprint: typically <2GB for 24h capture

### Resource Usage

Typical resource footprint:
  - capture (root): CPU <2%, RSS 20-50MB
  - locald (user): CPU <1%, RSS 30-80MB (DB growth)
  - ui_server (user): CPU <1%, RSS 10-20MB
  - analysis.db: 100MB-1GB (varies with incident count)

### Performance Baseline

  Smoke test (cold start):
  - Time to first incident: 2-5 seconds
  - Event throughput: 500-2000 events/sec (depends on audit load)
  - Deref latency: <100ms per incident
  - Index update latency: <50ms per segment

---

## Troubleshooting

### "ERROR: capture_macos_rotating requires root. Run with sudo."

  capture must run with root privileges to read /dev/auditpipe.
  If installed via LaunchD, this is automatic.

### "ERROR: Failed to open /dev/auditpipe: Permission denied"

  Audit daemon is not running. Check:
    $ launchctl list | grep auditd
    $ sudo launchctl kickstart -k system/com.apple.auditd

### No incidents are being created

  1. Verify audit is capturing events:
       $ praudit -l /dev/auditpipe | head -20
  
  2. Check locald logs:
       $ tail -50 /var/log/modela/locald.out.log
  
  3. Count raw events:
       $ sqlite3 /opt/modela/telemetry/analysis.db \
         "SELECT COUNT(*) FROM telemetry_events;"
  
  4. Verify playbooks are loaded:
       $ ls -la /opt/modela/playbooks/macos/

### API returns empty incidents list

  Check if edr_locald is running:
    $ launchctl list | grep locald
    $ curl http://127.0.0.1:8000/api/health

  Check analysis.db:
    $ sqlite3 /opt/modela/telemetry/analysis.db \
      "SELECT COUNT(*) FROM incidents;"

---

## Further Documentation

  - **PRODUCTION_PROOF_COMMANDS.txt**: Hard commands with expected outputs
  - **MODEL_A_FILE_MAP.txt**: Architecture, call chains, file organization
  - **scripts/macos/run_detection_matrix.sh**: Detection test harness + gap reporting
  - **playbooks/macos/*.yaml**: Individual playbook definitions

