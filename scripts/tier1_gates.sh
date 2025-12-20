#!/bin/bash
# scripts/tier1_gates.sh
# Validate that Tier-1 binaries compile cleanly (optional features).
# Tier-1: fingerprint_gen, edr_wrap
# These are optional and may depend on Tier-0 infrastructure.

set -e

echo "=== TIER-1 GATE VALIDATION ==="

echo "cargo check --bin fingerprint_gen..."
cargo check --bin fingerprint_gen

echo "cargo check --bin edr_wrap..."
cargo check --bin edr_wrap

echo ""
echo "=== TIER-1 GATE: PASSED ==="
