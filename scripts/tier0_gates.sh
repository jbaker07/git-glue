#!/bin/bash
# scripts/tier0_gates.sh
# Validate that Tier-0 binaries compile cleanly (strict gate for shipping lanes).
# Tier-0: forensic_hooks, capture_macos_rotating, edr_locald, ui_server
# These must compile without new feature flags.

set -e

echo "=== TIER-0 GATE VALIDATION ==="

# Library
echo "cargo check --lib..."
cargo check --lib

# Tier-0 binaries (shipped in production)
echo "cargo check --bin forensic_hooks..."
cargo check --bin forensic_hooks

echo "cargo check --bin capture_macos_rotating..."
cargo check --bin capture_macos_rotating

echo "cargo check --bin edr_locald..."
cargo check --bin edr_locald

echo "cargo check --bin ui_server..."
cargo check --bin ui_server

echo ""
echo "=== TIER-0 GATE: PASSED ==="
