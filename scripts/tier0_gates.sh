#!/bin/bash
# scripts/tier0_gates.sh
# Validate that Tier-0 binary compiles cleanly (strict gate for shipping lanes).
# Tier-0: forensic_hooks (primary production detection server)
# Must compile without new feature flags.

set -e

echo "=== TIER-0 GATE VALIDATION ==="

echo "cargo check --lib..."
cargo check --lib

echo "cargo check --bin forensic_hooks..."
cargo check --bin forensic_hooks

echo ""
echo "=== TIER-0 GATE: PASSED ==="
