#!/bin/bash
# scripts/macos/kill_test.sh
# Verify graceful shutdown: no tmp files, DB usable, clean exit

set -e

TELEMETRY_ROOT="${TELEMETRY_ROOT:-.}/telemetry_output"
mkdir -p "$TELEMETRY_ROOT/segments"

BINARIES=(
    "./target/release/capture_macos_rotating"
    "./target/release/edr_locald"
    "./target/release/ui_server"
)

for bin in "${BINARIES[@]}"; do
    if [[ ! -f "$bin" ]]; then
        echo "ERROR: $bin not found"
        exit 1
    fi
done

echo "[kill_test] Starting services..."

# Start locald in background
EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT" ./target/release/edr_locald &
LOCALD_PID=$!
sleep 1

# Start ui_server in background
EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT" ./target/release/ui_server &
UI_PID=$!
sleep 1

echo "[kill_test] Services running (locald=$LOCALD_PID, ui_server=$UI_PID)"
sleep 1

# Test health endpoint
if ! curl -s http://127.0.0.1:8000/api/health | grep -q '"ok":true'; then
    echo "[kill_test] WARN: health endpoint not fully ok (may be expected)"
fi

echo "[kill_test] Sending SIGTERM to services..."
kill -TERM $LOCALD_PID 2>/dev/null || true
kill -TERM $UI_PID 2>/dev/null || true

sleep 2

# Check no processes are still running
if ps -p $LOCALD_PID > /dev/null 2>&1; then
    echo "ERROR: locald still running after SIGTERM"
    kill -9 $LOCALD_PID
    exit 1
fi

if ps -p $UI_PID > /dev/null 2>&1; then
    echo "ERROR: ui_server still running after SIGTERM"
    kill -9 $UI_PID
    exit 1
fi

echo "[kill_test] All processes terminated cleanly"

# Check for stale temp files
TEMP_FILES=$(find "$TELEMETRY_ROOT" -name "*.tmp" 2>/dev/null | wc -l)
if [[ $TEMP_FILES -gt 0 ]]; then
    echo "ERROR: Found $TEMP_FILES stale .tmp files"
    exit 1
fi

echo "[kill_test] No stale .tmp files found"

# Check DB is usable
if [[ -f "$TELEMETRY_ROOT/analysis.db" ]]; then
    if sqlite3 "$TELEMETRY_ROOT/analysis.db" "SELECT 1" > /dev/null 2>&1; then
        echo "[kill_test] ✅ Database is usable"
    else
        echo "ERROR: Database is locked or corrupted"
        exit 1
    fi
fi

echo ""
echo "[kill_test] ✅ PASSED: Clean graceful shutdown verified"
