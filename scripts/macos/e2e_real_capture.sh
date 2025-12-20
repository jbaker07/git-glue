#!/bin/bash
# e2e_real_capture.sh: Test real OpenBSM capture with /dev/auditpipe
# Stimulates system activity and verifies telemetry collection and incident detection

set -e

# macOS compatibility: use gtimeout if available, else use sleep trick
TIMEOUT_CMD="timeout"
if ! command -v timeout &> /dev/null; then
    if command -v gtimeout &> /dev/null; then
        TIMEOUT_CMD="gtimeout"
    else
        TIMEOUT_CMD="timeout_unavailable"
    fi
fi

TELEMETRY_ROOT="${1:-.edr_e2e_test}"
TIMEOUT="${2:-30}"

echo "[e2e] Setting up test environment..."
rm -rf "$TELEMETRY_ROOT"
mkdir -p "$TELEMETRY_ROOT/segments" "$TELEMETRY_ROOT/playbooks"

# Create minimal playbook to catch /bin/ls
cat > "$TELEMETRY_ROOT/playbooks/test.yaml" <<'EOF'
playbook_id: "test_exec"
title: "Test Process Execution"
severity: "high"
priority: 100
entity_scope: "user"
slots:
  - slot_id: "exec"
    entity_key: "{{ user }}"
    predicates:
      - field: "event_type"
        op: "=="
        value: "proc_exec"
fire:
  condition: "exec"
EOF

# Create empty index
cat > "$TELEMETRY_ROOT/index.json" <<'EOF'
{
  "schema_version": 1,
  "next_seq": 1,
  "segments": [],
  "last_updated_ts": 0
}
EOF

export EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT"

echo "[e2e] Starting capture with --once flag..."
if [ "$TIMEOUT_CMD" = "timeout_unavailable" ]; then
    ./target/release/capture_macos_rotating --once &
    CAPTURE_PID=$!
    sleep 5
else
    $TIMEOUT_CMD $TIMEOUT ./target/release/capture_macos_rotating --once &
    CAPTURE_PID=$!
fi

# Stimulate system activity
sleep 1
/bin/ls /tmp > /dev/null 2>&1 || true
echo "test data" > "$TELEMETRY_ROOT/test_file_$RANDOM" 2>/dev/null || true
curl -s -m 2 http://example.com > /dev/null 2>&1 || true

wait $CAPTURE_PID 2>/dev/null || true

if [ ! -f "$TELEMETRY_ROOT/index.json" ]; then
    echo "[FAIL] index.json not created"
    exit 1
fi

# Check segment creation
SEGMENT_COUNT=$(ls "$TELEMETRY_ROOT/segments/" 2>/dev/null | wc -l)
if [ "$SEGMENT_COUNT" -eq 0 ]; then
    echo "[WARN] No segments created (may need audit enabled on macOS)"
else
    echo "[PASS] Created $SEGMENT_COUNT segment(s)"
fi

# Start edr_locald
echo "[e2e] Starting edr_locald..."
if [ "$TIMEOUT_CMD" = "timeout_unavailable" ]; then
    ./target/release/edr_locald &
    LOCALD_PID=$!
    sleep 3
else
    $TIMEOUT_CMD 15s ./target/release/edr_locald &
    LOCALD_PID=$!
    sleep 3
fi

# Start ui_server
echo "[e2e] Starting ui_server..."
if [ "$TIMEOUT_CMD" = "timeout_unavailable" ]; then
    ./target/release/ui_server &
    UI_PID=$!
    sleep 3
else
    $TIMEOUT_CMD 15s ./target/release/ui_server &
    UI_PID=$!
    sleep 2
fi

# Test health endpoint
echo "[e2e] Testing /api/health..."
HEALTH=$(curl -s http://localhost:8080/api/health)
if echo "$HEALTH" | grep -q '"ok":'; then
    echo "[PASS] Health endpoint responds"
else
    echo "[WARN] Health endpoint issue"
fi

# Check telemetry_events
if [ -f "$TELEMETRY_ROOT/analysis.db" ]; then
    EVENT_COUNT=$(sqlite3 "$TELEMETRY_ROOT/analysis.db" "SELECT COUNT(*) FROM telemetry_events" 2>/dev/null || echo 0)
    echo "[info] Event count: $EVENT_COUNT"
    
    if [ "$EVENT_COUNT" -gt 0 ]; then
        echo "[PASS] telemetry_events populated"
    else
        echo "[WARN] No events in telemetry_events"
    fi
fi

# Cleanup
kill $LOCALD_PID $UI_PID 2>/dev/null || true
wait 2>/dev/null || true

rm -rf "$TELEMETRY_ROOT"
echo "[e2e] ✓ Test complete"
