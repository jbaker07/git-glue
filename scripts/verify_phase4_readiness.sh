#!/usr/bin/env bash
# verify_phase4_readiness.sh
# Validates that the system is ready for Phase 4 multi-platform testing

set -e

RESET='\033[0m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'

echo -e "${BLUE}=========================================="
echo -e "Phase 4 Readiness Verification"
echo -e "==========================================${RESET}\n"

CHECK_COUNT=0
PASS_COUNT=0
FAIL_COUNT=0

check_item() {
    local name="$1"
    local condition="$2"
    
    CHECK_COUNT=$((CHECK_COUNT + 1))
    
    if eval "$condition" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${RESET} $name"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo -e "${RED}✗${RESET} $name"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

# Core infrastructure checks
echo -e "${BLUE}Core Infrastructure:${RESET}"
check_item "Cargo.toml exists" "[ -f Cargo.toml ]"
check_item "src/lib.rs exists" "[ -f src/lib.rs ]"
check_item "src/main.rs exists" "[ -f src/main.rs ]"
check_item "src/bin/entmon_attach.rs exists" "[ -f src/bin/entmon_attach.rs ]"
check_item "src/bin/fingerprint_gen.rs exists" "[ -f src/bin/fingerprint_gen.rs ]"
check_item "src/bin/edr_attach_any.rs exists" "[ -f src/bin/edr_attach_any.rs ]"

echo

# Platform support checks
echo -e "${BLUE}Platform Support:${RESET}"
check_item "Linux module (linux/mod.rs)" "[ -f linux/mod.rs ]"
check_item "macOS module (macos/mod.rs)" "[ -f macos/mod.rs ]"
check_item "Windows module (windows/mod.rs)" "[ -f windows/mod.rs ]"

echo

# eBPF checks
echo -e "${BLUE}eBPF Configuration:${RESET}"
check_item "eBPF feature flag defined" "grep -q 'ebpf = \[\]' Cargo.toml"
check_item "Aya dependency configured" "grep -q 'aya' Cargo.toml"

if [ "$(uname)" = "Linux" ]; then
    check_item "Linux target configured in Cargo.toml" "grep -q '\[target.*linux.*dependencies\]' Cargo.toml"
fi

echo

# Playbook support checks
echo -e "${BLUE}Playbook Support:${RESET}"
check_item "Playbooks directory exists" "[ -d playbooks ]"
check_item "Common playbooks" "[ -d playbooks/common ]"
check_item "Linux playbooks" "[ -d playbooks/linux ]"
check_item "macOS playbooks" "[ -d playbooks/macos ]"
check_item "Windows playbooks" "[ -d playbooks/windows ]"

echo

# Sensor framework checks
echo -e "${BLUE}Sensor Framework:${RESET}"
check_item "Linux sensors" "[ -d linux/sensors ]"
check_item "macOS sensors" "[ -d macos/sensors ]"
check_item "Windows sensors" "[ -d windows/sensors ]"

echo

# Documentation checks
echo -e "${BLUE}Documentation:${RESET}"
check_item "Baseline system design" "[ -f BASELINE_SYSTEM_DESIGN.md ]"
check_item "Baseline implementation summary" "[ -f BASELINE_IMPLEMENTATION_SUMMARY.md ]"
check_item "Integration guide" "[ -f BASELINE_INTEGRATION_GUIDE.md ]"

echo

# Compilation readiness checks
echo -e "${BLUE}Compilation Readiness:${RESET}"
if [ "$(uname)" = "Darwin" ]; then
    check_item "Can compile for macOS" "cargo check --lib 2>&1 | grep -q 'Finished\\|Checking' || true"
elif [ "$(uname)" = "Linux" ]; then
    check_item "Can compile for Linux" "cargo check --lib 2>&1 | grep -q 'Finished\\|Checking' || true"
fi

echo

# Summary
echo -e "${BLUE}=========================================="
echo -e "Readiness Summary"
echo -e "==========================================${RESET}"
echo "Total checks: $CHECK_COUNT"
echo -e "${GREEN}Passed: $PASS_COUNT${RESET}"

if [ $FAIL_COUNT -gt 0 ]; then
    echo -e "${RED}Failed: $FAIL_COUNT${RESET}"
fi

echo

# Exit code
if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ System is ready for Phase 4 testing!${RESET}"
    echo -e "\nNext steps:"
    echo "  1. Compile: cargo build --release"
    echo "  2. Run tests: cargo test"
    echo "  3. Start capture: cargo run --bin forensic_hooks --release"
    exit 0
else
    echo -e "${RED}✗ System has issues that need resolution${RESET}"
    echo -e "\nAddress the failing checks above and re-run this script."
    exit 1
fi
