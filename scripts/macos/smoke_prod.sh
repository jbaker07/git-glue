#!/bin/bash

# smoke_prod.sh: Minimal production smoke test for macOS Model A
# 
# Usage: scripts/macos/smoke_prod.sh [--keep-temp]
#
# Launches capture_macos_rotating, edr_locald, ui_server in temp TELEMETRY_ROOT
# Runs minimal stimulus, validates output, shuts down cleanly

set -e

KEEP_TEMP=false
if [[ "$1" == "--keep-temp" ]]; then
    KEEP_TEMP=true
fi

# Create temp telemetry root
TEMP_ROOT=$(mktemp -d)
export EDR_TELEMETRY_ROOT="$TEMP_ROOT"

echo "[smoke] TELEMETRY_ROOT=$TEMP_ROOT"
echo "[smoke] Building binaries..."

# Build (assume we're in git-glue root)
cargo build --release --bin capture_macos_rotating --bin edr_locald --bin ui_server 2>/dev/null || {
    echo "[ERROR] Build failed"
    exit 1
}

# Prepare playbooks
mkdir -p "$TEMP_ROOT/playbooks/macos"
cp playbooks/macos/*.yaml "$TEMP_ROOT/playbooks/macos/" 2>/dev/null || true

echo "[smoke] Starting daemons..."

# Launch daemons in background
sudo -E "$PWD/target/release/capture_macos_rotating" --telemetry-root "$TEMP_ROOT" > /tmp/capture.log 2>&1 &
CAPTURE_PID=$!

sleep 1

./target/release/edr_locald > /tmp/locald.log 2>&1 &
LOCALD_PID=$!

./target/release/ui_server > /tmp/ui.log 2>&1 &
UI_PID=$!

sleep 2

echo "[smoke] Stimulus: basic commands..."

# Reliable proc_exec
/bin/ls -la /etc/passwd > /dev/null 2>&1

# Reliable file_write: atomic write pattern (write + rename)
echo "modela_test_data" > /tmp/modela_smoke.tmp && mv /tmp/modela_smoke.tmp /tmp/modela_smoke.txt 2>/dev/null || true

# Also try a direct touch+write combination
touch /tmp/modela_touch.txt && echo "data" >> /tmp/modela_touch.txt 2>/dev/null || true

# Reliable net_connect: use curl with explicit IPv4
curl -4 -s https://example.com > /dev/null 2>&1 || true
# Fallback: try nc if available
which nc > /dev/null && nc -w 2 example.com 443 < /dev/null 2>/dev/null || true

sleep 3

echo ""
echo "[smoke] DATABASE COUNTS:"
COUNTS_SQL="SELECT event_type, COUNT(*) as count FROM telemetry_events GROUP BY event_type ORDER BY count DESC;"
COUNTS_OUTPUT=$(sqlite3 "$TEMP_ROOT/analysis.db" "$COUNTS_SQL" 2>/dev/null)

if [[ -z "$COUNTS_OUTPUT" ]]; then
    echo "  ERROR: No events in telemetry_events!"
    echo "  Last 20 events:"
    sqlite3 "$TEMP_ROOT/analysis.db" "SELECT ts_ms, event_type, COUNT(*) FROM telemetry_events GROUP BY ts_ms, event_type LIMIT 20;" 2>/dev/null || echo "  (empty)"
    TEST_PASS=false
else
    echo "$COUNTS_OUTPUT"
    
    # Extract counts
    PROC_EXEC=$(echo "$COUNTS_OUTPUT" | grep "proc_exec" | awk '{print $2}' || echo "0")
    FILE_WRITE=$(echo "$COUNTS_OUTPUT" | grep -E "file_write|file_create" | awk '{print $2}' | head -1 || echo "0")
    NET_CONNECT=$(echo "$COUNTS_OUTPUT" | grep "net_connect" | awk '{print $2}' || echo "0")
    
    PROC_EXEC=${PROC_EXEC:-0}
    FILE_WRITE=${FILE_WRITE:-0}
    NET_CONNECT=${NET_CONNECT:-0}
    
    echo ""
    echo "[smoke] VALIDATION:"
    echo "  proc_exec: $PROC_EXEC (need >= 1)"
    echo "  file_write: $FILE_WRITE (need >= 1)"
    echo "  net_connect: $NET_CONNECT (need >= 1)"
    
    # GATE: fail if any critical event type is missing
    if [[ $PROC_EXEC -lt 1 ]] || [[ $FILE_WRITE -lt 1 ]] || [[ $NET_CONNECT -lt 1 ]]; then
        echo ""
        echo "[smoke] ❌ FAILED: Missing required event types"
        echo "  Debug: Last 30 rows of telemetry_events:"
        sqlite3 "$TEMP_ROOT/analysis.db" "SELECT ts_ms, event_type, event_json FROM telemetry_events ORDER BY ts_ms DESC LIMIT 30;" 2>/dev/null | cut -c1-100
        TEST_PASS=false
    else
        echo ""
        echo "[smoke] ✅ PASSED: All event types present"
        TEST_PASS=true
    fi
fi

echo ""
echo "[smoke] INCIDENTS:"
sqlite3 "$TEMP_ROOT/analysis.db" "SELECT incident_id, playbook_id, severity FROM incidents LIMIT 5;" 2>/dev/null || echo "  (no incidents)"

echo ""
echo "[smoke] API INCIDENTS (first 3):"
curl -s http://127.0.0.1:8000/api/incidents | jq '.incidents[0:3]' 2>/dev/null || echo "  (API unavailable)"

# If there are incidents, show first explain
FIRST_INCIDENT=$(sqlite3 "$TEMP_ROOT/analysis.db" "SELECT incident_id FROM incidents LIMIT 1;" 2>/dev/null)
if [[ -n "$FIRST_INCIDENT" ]]; then
    echo ""
    echo "[smoke] FIRST INCIDENT EXPLAIN:"
    curl -s "http://127.0.0.1:8000/api/incidents/$FIRST_INCIDENT/explain" | jq '{incident_id, playbook_id, narrative, timeline: (.timeline | first)}' 2>/dev/null || echo "  (explain failed)"
fi

echo ""
echo "[smoke] Shutting down..."

# Kill daemons
kill $CAPTURE_PID 2>/dev/null || true
kill $LOCALD_PID 2>/dev/null || true
kill $UI_PID 2>/dev/null || true

sleep 1

# Cleanup
if [[ "$KEEP_TEMP" == "false" ]]; then
    rm -rf "$TEMP_ROOT"
    echo "[smoke] Cleaned up temp root"
else
    echo "[smoke] Kept temp root: $TEMP_ROOT"
fi

echo "[smoke] DONE"
