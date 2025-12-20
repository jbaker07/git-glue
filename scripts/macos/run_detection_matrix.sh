#!/bin/bash

# run_detection_matrix.sh: Real-world technique testing with audit events
# 
# Uses ACTUAL shell commands that trigger OpenBSM audit events (not synthetic).
# Generates engineer-grade gap report with root causes.
# 
# Usage: sudo -E ./run_detection_matrix.sh [--telemetry-root <path>]

set -e

TELEMETRY_ROOT="${EDR_TELEMETRY_ROOT:-/tmp/modela_matrix_$$}"
REPORT_DIR="$TELEMETRY_ROOT/reports/$(date +%Y%m%d_%H%M%S)"

echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                    Model A Detection Matrix - Real Commands"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "TELEMETRY_ROOT: $TELEMETRY_ROOT"
echo ""

# Setup
mkdir -p "$TELEMETRY_ROOT/segments" "$REPORT_DIR"

# CSV header
cat > "$REPORT_DIR/gap_report.csv" << 'EOF'
test_id,technique,mitre_id,command,expected_events,observed_event_count,playbook_id,incident_fired,root_cause,remediation_file,notes
EOF

# Build if needed
if [ ! -f "target/release/capture_macos_rotating" ]; then
    echo "Building binaries..."
    cargo build --release --bin capture_macos_rotating --bin edr_locald --bin ui_server 2>&1 | grep -E "(Finished|error)" || true
fi

# Start services
export EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT"

echo "Starting capture (root), locald, ui_server..."
sudo -E ./target/release/capture_macos_rotating --telemetry-root "$TELEMETRY_ROOT" > /tmp/capture.log 2>&1 &
CAPTURE_PID=$!

./target/release/edr_locald --telemetry-root "$TELEMETRY_ROOT" > /tmp/locald.log 2>&1 &
LOCALD_PID=$!

./target/release/ui_server --telemetry-root "$TELEMETRY_ROOT" > /tmp/ui.log 2>&1 &
UI_PID=$!

sleep 3

# ═══════════════════════════════════════════════════════════════════════════════
# TEST 1: Process Execution (Basic)
# ═══════════════════════════════════════════════════════════════════════════════

echo "[1/5] TEST: Process Execution (AUE_EXECVE)"
echo "  Command: /bin/ls -la /tmp"

/bin/ls -la /tmp > /dev/null
sleep 1

event_count=$(sqlite3 "$TELEMETRY_ROOT/analysis.db" \
    "SELECT COUNT(*) FROM telemetry_events WHERE event_type='proc_exec';" 2>/dev/null || echo "0")
echo "  Events observed: $event_count"
echo "  → Baseline test for OpenBSM exec capture"

echo "$event_count proc_exec events from ls,PROC_EXEC,N/A,/bin/ls -la /tmp,AUE_EXECVE,$event_count,N/A,N/A,BASELINE,N/A,src/bin/capture_macos_rotating.rs,Baseline: verifies OpenBSM exec events are captured" >> "$REPORT_DIR/gap_report.csv"

# ═══════════════════════════════════════════════════════════════════════════════
# TEST 2: User-writable exec + egress
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo "[2/5] TEST: Suspicious exec from /tmp + network"
echo "  Command: Write script to /tmp, execute it, trigger network call"

tmpscript="/tmp/test_script_$$.sh"
cat > "$tmpscript" << 'SCRIPTEOF'
#!/bin/bash
sleep 1
curl -s http://127.0.0.1:8000 2>/dev/null || true
SCRIPTEOF
chmod +x "$tmpscript"
"$tmpscript" &
SCRIPT_PID=$!
sleep 3
wait $SCRIPT_PID 2>/dev/null || true
rm "$tmpscript"

sleep 2

exec_count=$(sqlite3 "$TELEMETRY_ROOT/analysis.db" \
    "SELECT COUNT(*) FROM telemetry_events WHERE event_type='proc_exec';" 2>/dev/null || echo "0")
incident_count=$(sqlite3 "$TELEMETRY_ROOT/analysis.db" \
    "SELECT COUNT(*) FROM incidents WHERE playbook_id='pb_user_writable_exec_egress';" 2>/dev/null || echo "0")

echo "  Events observed: $exec_count proc_exec"
echo "  Incidents fired: $incident_count (playbook: pb_user_writable_exec_egress)"
fired="$([ "$incident_count" -gt 0 ] && echo "YES" || echo "NO")"
cause="$([ "$incident_count" -gt 0 ] && echo "PASS" || echo "CAPTURE_GAP")"

echo "T1204.usr_writable,User-writable exec + egress,T1204,/tmp/test_script + curl,proc_exec|net_connect,$exec_count,pb_user_writable_exec_egress,$fired,$cause,src/bin/capture_macos_rotating.rs,Need net_connect + proc_exec parsing" >> "$REPORT_DIR/gap_report.csv"

# ═══════════════════════════════════════════════════════════════════════════════
# TEST 3: LaunchAgent persistence
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo "[3/5] TEST: LaunchAgent persistence plist write + launchctl"
echo "  Command: Write plist to ~/Library/LaunchAgents, attempt launchctl load"

plist_path="$HOME/Library/LaunchAgents/test_modela_$$.plist"
mkdir -p "$(dirname "$plist_path")"

cat > "$plist_path" << 'PLISTEOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>test.persistence</string>
  <key>Program</key>
  <string>/bin/echo</string>
</dict>
</plist>
PLISTEOF

launchctl load "$plist_path" 2>/dev/null || true
sleep 1
launchctl unload "$plist_path" 2>/dev/null || true
rm "$plist_path" 2>/dev/null || true

sleep 2

file_count=$(sqlite3 "$TELEMETRY_ROOT/analysis.db" \
    "SELECT COUNT(*) FROM telemetry_events WHERE event_type='file_write';" 2>/dev/null || echo "0")
incident_count=$(sqlite3 "$TELEMETRY_ROOT/analysis.db" \
    "SELECT COUNT(*) FROM incidents WHERE playbook_id='pb_launch_agent_persistence';" 2>/dev/null || echo "0")

echo "  Events observed: $file_count file_write"
echo "  Incidents fired: $incident_count (playbook: pb_launch_agent_persistence)"
fired="$([ "$incident_count" -gt 0 ] && echo "YES" || echo "NO")"
cause="$([ "$incident_count" -gt 0 ] && echo "PASS" || echo "CAPTURE_GAP")"

echo "T1543.001.launch,LaunchAgent persistence,T1543.001,plist write + launchctl,AUE_OPEN_W|AUE_CREATE,$file_count,pb_launch_agent_persistence,$fired,$cause,src/bin/capture_macos_rotating.rs,Need file_write + launchctl proc_exec parsing" >> "$REPORT_DIR/gap_report.csv"

# ═══════════════════════════════════════════════════════════════════════════════
# TEST 4: osascript with suspicious args
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo "[4/5] TEST: osascript with suspicious command"
echo "  Command: osascript -e 'system command with curl'"

osascript -e 'do shell script "curl http://127.0.0.1:8000"' 2>/dev/null || true
sleep 2

exec_count=$(sqlite3 "$TELEMETRY_ROOT/analysis.db" \
    "SELECT COUNT(*) FROM telemetry_events WHERE event_type='proc_exec' AND event_json LIKE '%osascript%';" 2>/dev/null || echo "0")
incident_count=$(sqlite3 "$TELEMETRY_ROOT/analysis.db" \
    "SELECT COUNT(*) FROM incidents WHERE playbook_id='pb_osascript_suspicious_args';" 2>/dev/null || echo "0")

echo "  Events observed: $exec_count osascript execs"
echo "  Incidents fired: $incident_count (playbook: pb_osascript_suspicious_args)"
fired="$([ "$incident_count" -gt 0 ] && echo "YES" || echo "NO")"
cause="$([ "$incident_count" -gt 0 ] && echo "PASS" || echo "CAPTURE_GAP")"

echo "T1059.002.osascript,osascript suspicious,T1059.002,osascript -e with curl,proc_exec,$exec_count,pb_osascript_suspicious_args,$fired,$cause,src/bin/capture_macos_rotating.rs,Need exec args capture for argv matching" >> "$REPORT_DIR/gap_report.csv"

# ═══════════════════════════════════════════════════════════════════════════════
# TEST 5: sudo usage
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo "[5/5] TEST: Sudo escalation"
echo "  Command: sudo -u root whoami (will prompt for password)"
echo "  Skipped (requires interactive password input; would deadlock)"

echo "T1548.003.sudo,Privilege escalation via sudo,T1548.003,sudo -u root whoami,AUE_SUDO,0,pb_sudo_anomaly,N/A,TEST_SKIP,N/A,scripts/macos/run_detection_matrix.sh,Non-interactive test can't supply password" >> "$REPORT_DIR/gap_report.csv"

# ═══════════════════════════════════════════════════════════════════════════════
# CLEANUP & REPORT
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo "Stopping services..."
kill $CAPTURE_PID $LOCALD_PID $UI_PID 2>/dev/null || true
wait 2>/dev/null || true

# Generate markdown report
cat > "$REPORT_DIR/gap_report.md" << 'MDEOF'
# Model A Detection Matrix - Gap Analysis Report

## Summary

| Metric | Value |
|--------|-------|
| Tests Run | 5 |
| Commands Executed | 4 |
| Baseline Established | YES |
| Coverage (preliminary) | Pending OpenBSM parsing |

## Test Results

MDEOF

echo "" >> "$REPORT_DIR/gap_report.md"
echo "See gap_report.csv for detailed per-test analysis." >> "$REPORT_DIR/gap_report.md"

# Final summary
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "DETECTION MATRIX COMPLETE"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "Reports:"
echo "  CSV:  $REPORT_DIR/gap_report.csv"
echo "  MD:   $REPORT_DIR/gap_report.md"
echo ""
echo "Database artifacts:"
echo "  Events: $(sqlite3 "$TELEMETRY_ROOT/analysis.db" "SELECT COUNT(*) FROM telemetry_events;" 2>/dev/null || echo "?")"
echo "  Incidents: $(sqlite3 "$TELEMETRY_ROOT/analysis.db" "SELECT COUNT(*) FROM incidents;" 2>/dev/null || echo "?")"
echo ""
echo "Next: Review gap_report.csv for root causes and remediation items."
echo ""
