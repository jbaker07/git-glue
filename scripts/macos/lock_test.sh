#!/bin/bash
# P0: Test concurrent daemon safety - single locald lock
# Verifies that second locald instance fails with lock contention

set -e

TELEMETRY_ROOT="${1:-.telemetry_lock_test}"

echo "[test] Setting up lock test..."
rm -rf "$TELEMETRY_ROOT"
mkdir -p "$TELEMETRY_ROOT/segments"
mkdir -p "$TELEMETRY_ROOT/playbooks"

# Create minimal index
cat > "$TELEMETRY_ROOT/index.json" <<'EOF'
{
  "schema_version": 1,
  "next_seq": 1,
  "segments": [],
  "last_updated_ts": 0
}
EOF

# Create empty playbooks dir (required by locald)
touch "$TELEMETRY_ROOT/playbooks/.gitkeep"

export EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT"

echo "[test] Starting first edr_locald (should acquire lock)..."
timeout 10s ./target/release/edr_locald &
LOCALD_PID1=$!
sleep 2  # Let it acquire lock

echo "[test] Starting second edr_locald (should fail with lock error)..."
if timeout 5s ./target/release/edr_locald 2>&1 | grep -q "Failed to acquire lock\|lock contended"; then
    echo "[PASS] Second locald correctly failed due to lock contention"
    SECOND_EXIT=0
else
    echo "[FAIL] Second locald should have failed with lock error"
    SECOND_EXIT=1
fi

# Clean up first instance
kill $LOCALD_PID1 2>/dev/null || true
wait $LOCALD_PID1 2>/dev/null || true

# Check for lock contention metric
if [ -f "$TELEMETRY_ROOT/analysis.db" ]; then
    sqlite3 "$TELEMETRY_ROOT/analysis.db" \
        "SELECT metric_name, count FROM metrics_rollup WHERE metric_name = 'locald_lock_contended'" \
        | grep -q locald_lock_contended && \
    echo "[PASS] Lock contention metric recorded" || \
    echo "[WARN] Metric not recorded (timing)"
fi

rm -rf "$TELEMETRY_ROOT"

if [ $SECOND_EXIT -eq 0 ]; then
    echo "[test] ✓ Lock test passed"
    exit 0
else
    echo "[test] ✗ Lock test failed"
    exit 1
fi
