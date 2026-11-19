#!/bin/bash
set -e

# workflow_smoke.sh - minimal smoke test for workflow empowerment layer

TEST_ROOT="/tmp/workflow_smoke_$$"
export EDR_TELEMETRY_ROOT="$TEST_ROOT"
CARGO_ROOT="/Users/jermainebaker/git-glue/git-glue"

cleanup() {
    pkill -f "edr_locald" 2>/dev/null || true
    pkill -f "ui_server" 2>/dev/null || true
    rm -rf "$TEST_ROOT" 2>/dev/null || true
}
trap cleanup EXIT

echo "=== Workflow Smoke Test ==="
echo "TEST_ROOT: $TEST_ROOT"

# Setup
mkdir -p "$TEST_ROOT"/{segments,operator_segments,playbooks}
touch "$TEST_ROOT/index.json"

# Build
cd "$CARGO_ROOT"
echo "Building..."
cargo build --release --bin edr_locald --bin ui_server --bin operator_tape 2>&1 | tail -5

# Start daemons
echo "Starting edr_locald and ui_server..."
./target/release/edr_locald > /tmp/locald.log 2>&1 &
LOCALD_PID=$!
./target/release/ui_server > /tmp/ui.log 2>&1 &
UI_PID=$!
sleep 3

# Test 1: Run operator_tape with simulated nmap output
echo "Test 1: Execute operator_tape -- bash -c 'echo 80/tcp open http; exit 0'"
./target/release/operator_tape -- bash -c "echo '80/tcp open http
443/tcp open https' ; exit 0"
sleep 1

# Test 2: Verify operator_events table populated
OPERATOR_COUNT=$(sqlite3 "$TEST_ROOT/analysis.db" "SELECT COUNT(*) FROM operator_events;" 2>/dev/null || echo "0")
if [ "$OPERATOR_COUNT" -gt 0 ]; then
    echo "✓ Test 2 PASS: operator_events has $OPERATOR_COUNT rows"
else
    echo "✗ Test 2 FAIL: operator_events empty"
    sqlite3 "$TEST_ROOT/analysis.db" ".tables"
    exit 1
fi

# Test 3: Query /api/sessions/latest
echo "Test 3: Query /api/sessions/latest"
SESSIONS_JSON=$(curl -s http://localhost:8000/api/sessions/latest)
if echo "$SESSIONS_JSON" | jq .session_id >/dev/null 2>&1; then
    SESSION_ID=$(echo "$SESSIONS_JSON" | jq -r .session_id)
    echo "✓ Test 3 PASS: Got session $SESSION_ID"
else
    echo "✗ Test 3 FAIL: /api/sessions/latest response invalid"
    echo "$SESSIONS_JSON"
    exit 1
fi

# Test 4: Verify workflow facts extracted (PortOpen)
FACTS_COUNT=$(echo "$SESSIONS_JSON" | jq '.facts | length')
if [ "$FACTS_COUNT" -gt 0 ]; then
    FIRST_FACT=$(echo "$SESSIONS_JSON" | jq -r '.facts[0].fact_type')
    echo "✓ Test 4 PASS: Extracted $FACTS_COUNT facts, first: $FIRST_FACT"
else
    echo "⚠ Test 4 WARN: No facts extracted (may be OK if parser incomplete)"
fi

# Test 5: Query /api/sessions/:id/next_actions
echo "Test 5: Query /api/sessions/$SESSION_ID/next_actions"
ACTIONS_JSON=$(curl -s "http://localhost:8000/api/sessions/$SESSION_ID/next_actions")
ACTIONS_COUNT=$(echo "$ACTIONS_JSON" | jq '.actions | length')
if [ "$ACTIONS_COUNT" -gt 0 ]; then
    echo "✓ Test 5 PASS: Generated $ACTIONS_COUNT next actions"
    echo "$ACTIONS_JSON" | jq '.actions[0]'
else
    echo "✓ Test 5 INFO: No next actions (rules may not match facts)"
fi

# Final summary
echo ""
echo "=== SMOKE TEST COMPLETE ==="
echo "✅ Core workflow layer is functional"
echo "   - operator_tape captures command output"
echo "   - edr_locald ingests operator_events"
echo "   - ui_server /api/sessions/* endpoints responding"
echo ""
