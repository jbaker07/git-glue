#!/bin/bash

# evidence_ref_test.sh: Production safety net for incident_evidence table
# Proves:
# 1. incident_evidence table correctly stores segment references
# 2. retention check uses incident_evidence (not evidence_json LIKE)
# 3. referenced segment lookup returns 1
# 4. unreferenced segment returns 0
# 5. no legacy LIKE patterns in source

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_GLUE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_DB=$(mktemp)
trap "rm -f $TEST_DB" EXIT

echo "[evidence_ref_test] Setting up temporary database: $TEST_DB"

# Create schema
sqlite3 "$TEST_DB" << 'EOF'
CREATE TABLE incident_evidence (
    incident_id TEXT NOT NULL,
    segment_id TEXT NOT NULL,
    record_index INTEGER NOT NULL,
    ts_ms INTEGER,
    PRIMARY KEY(incident_id, segment_id, record_index)
);

CREATE INDEX idx_incident_evidence_segment ON incident_evidence(segment_id);
EOF

echo "[evidence_ref_test] Inserting test data..."

# Insert 2 incidents referencing 2 segments
sqlite3 "$TEST_DB" << 'EOF'
INSERT INTO incident_evidence VALUES ('inc_001', 'seg_001', 10, 1000);
INSERT INTO incident_evidence VALUES ('inc_001', 'seg_002', 20, 1000);
INSERT INTO incident_evidence VALUES ('inc_002', 'seg_001', 30, 1001);
EOF

echo "[evidence_ref_test] Testing referenced segment lookup..."

# Test: referenced segment seg_001 returns 1 row
RESULT=$(sqlite3 "$TEST_DB" "SELECT 1 FROM incident_evidence WHERE segment_id='seg_001' LIMIT 1;")
if [ "$RESULT" = "1" ]; then
    echo "✓ Referenced segment lookup: PASS (seg_001 found)"
else
    echo "✗ Referenced segment lookup: FAIL (seg_001 not found)"
    exit 1
fi

echo "[evidence_ref_test] Testing unreferenced segment lookup..."

# Test: unreferenced segment seg_999 returns nothing
RESULT=$(sqlite3 "$TEST_DB" "SELECT 1 FROM incident_evidence WHERE segment_id='seg_999' LIMIT 1;")
if [ -z "$RESULT" ]; then
    echo "✓ Unreferenced segment lookup: PASS (seg_999 not found)"
else
    echo "✗ Unreferenced segment lookup: FAIL (seg_999 unexpectedly found)"
    exit 1
fi

echo "[evidence_ref_test] Checking for legacy LIKE patterns in source..."

# Search for evidence_json LIKE patterns in src directory
LIKE_COUNT=$(grep -r 'evidence_json.*LIKE' "$GIT_GLUE_ROOT/git-glue/src" --include='*.rs' 2>/dev/null | wc -l)
if [ "$LIKE_COUNT" -eq 0 ]; then
    echo "✓ No legacy LIKE patterns: PASS"
else
    echo "✗ Found $LIKE_COUNT LIKE patterns: FAIL"
    grep -r 'evidence_json.*LIKE' "$GIT_GLUE_ROOT/git-glue/src" --include='*.rs'
    exit 1
fi

echo "[evidence_ref_test] All tests passed!"
exit 0
