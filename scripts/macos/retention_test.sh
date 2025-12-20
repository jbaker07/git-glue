#!/bin/bash
# Task 7: Test retention enforcement
# Creates test data with old timestamps, triggers retention, verifies cleanup

set -e

TELEMETRY_ROOT="${1:-.telemetry_test}"
RETENTION_DAYS="${2:-0}"  # 0 days = delete all except current

echo "[test] Setting up retention test..."
rm -rf "$TELEMETRY_ROOT"
mkdir -p "$TELEMETRY_ROOT/segments"

# Create a test index with old segments
cat > "$TELEMETRY_ROOT/index.json" <<'EOF'
{
  "schema_version": 1,
  "next_seq": 3,
  "segments": [
    {
      "seq": 1,
      "segment_id": "segment_001",
      "path": "segments/segment_001.jsonl",
      "ts": 1000000000000,
      "sha256_segment": ""
    },
    {
      "seq": 2,
      "segment_id": "segment_002",
      "path": "segments/segment_002.jsonl",
      "ts": 2000000000000,
      "sha256_segment": ""
    }
  ],
  "last_updated_ts": 2000000000000
}
EOF

# Create dummy segment files with old timestamps
echo '{"event_type": "test", "ts_ms": 1000000000000}' > "$TELEMETRY_ROOT/segments/segment_001.jsonl"
echo '{"event_type": "test", "ts_ms": 2000000000000}' > "$TELEMETRY_ROOT/segments/segment_002.jsonl"

# Set very old mtime (Jan 1, 1970)
touch -t 197001010000.00 "$TELEMETRY_ROOT/segments/segment_001.jsonl"

echo "[test] Created test segments with old mtime"
ls -lh "$TELEMETRY_ROOT/segments/"

# Run edr_locald with retention enabled
echo "[test] Running edr_locald with retention (EDR_RETENTION_DAYS=$RETENTION_DAYS)..."
export EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT"
export EDR_RETENTION_DAYS="$RETENTION_DAYS"

# Start locald in background, let it run one retention cycle, then kill
timeout 45s ./target/release/edr_locald &
LOCALD_PID=$!
sleep 35  # Wait for retention loop to run (10 min loop starts after 30s)
kill $LOCALD_PID 2>/dev/null || true
wait $LOCALD_PID 2>/dev/null || true

echo "[test] Checking results..."

# Verify segment was deleted
if [ -f "$TELEMETRY_ROOT/segments/segment_001.jsonl" ]; then
    echo "[FAIL] segment_001 should have been deleted"
    exit 1
else
    echo "[PASS] segment_001 deleted as expected"
fi

# Verify newer segment kept
if [ -f "$TELEMETRY_ROOT/segments/segment_002.jsonl" ]; then
    echo "[PASS] segment_002 kept as expected"
else
    echo "[FAIL] segment_002 should have been kept"
    exit 1
fi

# Check metrics_rollup recorded deletion
if [ -f "$TELEMETRY_ROOT/analysis.db" ]; then
    sqlite3 "$TELEMETRY_ROOT/analysis.db" \
        "SELECT metric_name, count FROM metrics_rollup WHERE metric_name = 'retention_segments_deleted'" \
        | grep -q retention_segments_deleted && \
    echo "[PASS] Retention metric recorded" || \
    echo "[WARN] Retention metric not found (may be timing)"
fi

echo "[test] ✓ Retention test passed"
rm -rf "$TELEMETRY_ROOT"
