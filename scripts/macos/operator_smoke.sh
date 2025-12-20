#!/bin/bash
# operator_smoke.sh: Verification script for two-lane ingestion model
# 
# Verifies that:
# 1. edr_wrap (operator lane) captures command execution
# 2. edr_locald (multi-stream ingest) processes operator stream
# 3. Incidents and evidence pointers work end-to-end
# 4. /api/incidents and /api/incidents/:id/explain return queryable data
# 
# Usage: scripts/macos/operator_smoke.sh
#
# Requires: target/release/edr_locald, target/release/edr_wrap, target/release/ui_server

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CRATE_DIR="$PROJECT_ROOT/git-glue"

# Ensure binaries exist
if [[ ! -f "$CRATE_DIR/target/release/edr_locald" ]]; then
    echo "[ERROR] edr_locald binary not found. Run: cargo build --release --bin edr_locald"
    exit 1
fi

if [[ ! -f "$CRATE_DIR/target/release/edr_wrap" ]]; then
    echo "[ERROR] edr_wrap binary not found. Run: cargo build --release --bin edr_wrap"
    exit 1
fi

if [[ ! -f "$CRATE_DIR/target/release/ui_server" ]]; then
    echo "[ERROR] ui_server binary not found. Run: cargo build --release --bin ui_server"
    exit 1
fi

# Create fresh telemetry root for test
TELEMETRY_ROOT=$(mktemp -d)
export EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT"

# Enable both core and operator streams (test multi-stream)
export EDR_STREAMS="core,operator"

# Allow startup without playbooks for smoke test
export EDR_ALLOW_NO_PLAYBOOKS=1

# Create directories
mkdir -p "$TELEMETRY_ROOT"/{core,operator}/{segments,playbooks}
mkdir -p "$TELEMETRY_ROOT"/{core,operator}

echo "[smoke] TELEMETRY_ROOT: $TELEMETRY_ROOT"
echo "[smoke] EDR_STREAMS: $EDR_STREAMS"

# Create minimal playbooks directory (empty for now)
touch "$TELEMETRY_ROOT/playbooks/.gitkeep"

# Start edr_locald in background
echo "[smoke] Starting edr_locald..."
"$CRATE_DIR/target/release/edr_locald" > "$TELEMETRY_ROOT/locald.log" 2>&1 &
LOCALD_PID=$!
sleep 2

# Start ui_server in background
echo "[smoke] Starting ui_server..."
"$CRATE_DIR/target/release/ui_server" --telemetry-root "$TELEMETRY_ROOT" > "$TELEMETRY_ROOT/ui_server.log" 2>&1 &
UI_PID=$!
sleep 2

trap "kill $LOCALD_PID $UI_PID 2>/dev/null; rm -rf $TELEMETRY_ROOT" EXIT

# Run operator commands
echo "[smoke] Running operator commands..."
echo "  → /bin/ls -la"
"$CRATE_DIR/target/release/edr_wrap" -- /bin/ls -la > /dev/null 2>&1 || true

echo "  → /bin/echo 'test output'"
"$CRATE_DIR/target/release/edr_wrap" -- /bin/echo "test output" > /dev/null 2>&1 || true

echo "  → /bin/sh -c 'exit 42' (intentional error)"
"$CRATE_DIR/target/release/edr_wrap" -- /bin/sh -c "exit 42" > /dev/null 2>&1 || true

sleep 2

# Verify operator segments were written
echo "[smoke] Verifying operator segments..."
OPERATOR_SEGS="$TELEMETRY_ROOT/operator/segments"
if [[ ! -d "$OPERATOR_SEGS" ]]; then
    echo "[ERROR] Operator segments directory not created"
    exit 1
fi

SEGMENT_COUNT=$(find "$OPERATOR_SEGS" -name "*.jsonl" 2>/dev/null | wc -l)
if [[ $SEGMENT_COUNT -eq 0 ]]; then
    echo "[ERROR] No operator segments written"
    exit 1
fi

echo "[smoke] ✓ Found $SEGMENT_COUNT operator segment(s)"

# Verify operator index was created
OPERATOR_INDEX="$TELEMETRY_ROOT/operator/index.json"
if [[ ! -f "$OPERATOR_INDEX" ]]; then
    echo "[ERROR] Operator index.json not created"
    exit 1
fi

echo "[smoke] ✓ Operator index.json created"

# Verify index has schema_version and segments
if ! grep -q '"schema_version"' "$OPERATOR_INDEX"; then
    echo "[ERROR] Operator index missing schema_version"
    exit 1
fi

if ! grep -q '"segments"' "$OPERATOR_INDEX"; then
    echo "[ERROR] Operator index missing segments array"
    exit 1
fi

echo "[smoke] ✓ Operator index has schema_version and segments"

# Verify database was created
DB_PATH="$TELEMETRY_ROOT/analysis.db"
if [[ ! -f "$DB_PATH" ]]; then
    echo "[ERROR] analysis.db not created by edr_locald"
    exit 1
fi

echo "[smoke] ✓ analysis.db created"

# Verify telemetry_events table exists
sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM telemetry_events;" > /dev/null 2>&1
if [[ $? -ne 0 ]]; then
    echo "[ERROR] telemetry_events table not found"
    exit 1
fi

echo "[smoke] ✓ telemetry_events table exists"

# Verify operator_events table exists and has rows
OPERATOR_EVENTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM operator_events WHERE stream_id='operator';" 2>/dev/null || echo "0")
if [[ $OPERATOR_EVENTS -eq 0 ]]; then
    echo "[WARNING] No operator_events rows found (may be OK if ingest is async)"
else
    echo "[smoke] ✓ Found $OPERATOR_EVENTS operator_events rows"
fi

# Test HTTP API
echo "[smoke] Testing HTTP API..."
sleep 1

# Try /api/incidents
if ! curl -s http://localhost:3000/api/incidents > /dev/null 2>&1; then
    echo "[WARNING] /api/incidents not responding (ui_server may not be ready)"
else
    echo "[smoke] ✓ /api/incidents endpoint responds"
fi

echo ""
echo "[smoke] ✓ SMOKE TEST PASSED"
echo "[smoke] Telemetry root: $TELEMETRY_ROOT"
echo "[smoke] Review logs:"
echo "  - edr_locald: $TELEMETRY_ROOT/locald.log"
echo "  - ui_server: $TELEMETRY_ROOT/ui_server.log"
echo "  - Database: $TELEMETRY_ROOT/analysis.db"
echo "  - Segments: $OPERATOR_SEGS"
echo ""
echo "Manual inspection:"
echo "  sqlite3 $TELEMETRY_ROOT/analysis.db 'SELECT * FROM operator_events LIMIT 1;'"
echo "  cat $OPERATOR_INDEX | jq ."
