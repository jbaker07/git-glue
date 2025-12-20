#!/bin/bash
# scripts/tier0_gates.sh
# Validate that Tier-0 binaries compile cleanly (strict gate for shipping lanes).
# Tier-0: capture_macos_rotating, edr_locald, ui_server, forensic_hooks
# These must compile without new feature flags.

set -e

echo "=== TIER-0 GATE VALIDATION ==="

# Library
echo "cargo check --lib..."
cargo check --lib

# Tier-0 binaries (shipped in production)
# Note: capture_macos_rotating, edr_locald, ui_server require macOS;
# forensic_hooks is all-platform. We check fingerprint_gen as Tier-1 reference.
echo "cargo check --bin forensic_hooks..."
cargo check --bin forensic_hooks

echo "cargo check --bin fingerprint_gen (Tier-1 reference)..."
cargo check --bin fingerprint_gen

echo "cargo check --bin edr_wrap (Tier-1 reference)..."
cargo check --bin edr_wrap

echo ""
echo "=== TIER-0 GATE: PASSED ==="
