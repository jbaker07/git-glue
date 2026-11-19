#!/bin/bash
# ==============================================================================
# Release Gates Script - tier2_gates.sh
# ==============================================================================
# 
# One-command verification that all credibility locks and core functionality
# are working before release.
#
# Usage:
#   ./scripts/tier2_gates.sh          # Run all tests
#   ./scripts/tier2_gates.sh --quick  # Skip slow tests (just compile + unit)
#   ./scripts/tier2_gates.sh --full   # Include clippy + fmt checks
#
# Exit codes:
#   0 = All gates pass
#   1 = Test failure
#   2 = Compilation failure
#   3 = Clippy/fmt issues (--full mode only)
#
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
QUICK_MODE=false
FULL_MODE=false

for arg in "$@"; do
    case $arg in
        --quick)
            QUICK_MODE=true
            ;;
        --full)
            FULL_MODE=true
            ;;
        --help|-h)
            echo "Release Gates Script"
            echo ""
            echo "Usage: $0 [--quick|--full]"
            echo ""
            echo "Options:"
            echo "  --quick  Skip slow tests, just compile + unit tests"
            echo "  --full   Include clippy and rustfmt checks"
            echo ""
            exit 0
            ;;
    esac
done

echo -e "${BLUE}============================================================${NC}"
echo -e "${BLUE}  Release Gates - Tier 2 Verification${NC}"
echo -e "${BLUE}============================================================${NC}"
echo ""

# Track timing
START_TIME=$(date +%s)

# ==============================================================================
# GATE 1: Compilation
# ==============================================================================
echo -e "${YELLOW}[1/5] Checking compilation...${NC}"

if ! cargo check --workspace 2>&1; then
    echo -e "${RED}✗ GATE 1 FAILED: Compilation errors${NC}"
    exit 2
fi
echo -e "${GREEN}✓ Gate 1: Compilation OK${NC}"
echo ""

# ==============================================================================
# GATE 2: Credibility Lock Tests
# ==============================================================================
echo -e "${YELLOW}[2/5] Running credibility lock tests...${NC}"

if ! cargo test --package edr-server --test credibility_lock_tests 2>&1; then
    echo -e "${RED}✗ GATE 2 FAILED: Credibility lock tests failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Gate 2: Credibility locks verified (22 tests)${NC}"
echo ""

# ==============================================================================
# GATE 3: Core Unit Tests
# ==============================================================================
echo -e "${YELLOW}[3/5] Running core unit tests...${NC}"

if $QUICK_MODE; then
    # Quick mode: just lib tests
    if ! cargo test --lib --workspace 2>&1; then
        echo -e "${RED}✗ GATE 3 FAILED: Unit tests failed${NC}"
        exit 1
    fi
else
    # Full test suite
    if ! cargo test --workspace 2>&1; then
        echo -e "${RED}✗ GATE 3 FAILED: Tests failed${NC}"
        exit 1
    fi
fi
echo -e "${GREEN}✓ Gate 3: Unit tests OK${NC}"
echo ""

# ==============================================================================
# GATE 4: Documentation Tests (skip in quick mode)
# ==============================================================================
if ! $QUICK_MODE; then
    echo -e "${YELLOW}[4/5] Running doc tests...${NC}"
    
    if ! cargo test --doc --workspace 2>&1; then
        echo -e "${RED}✗ GATE 4 FAILED: Doc tests failed${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Gate 4: Doc tests OK${NC}"
else
    echo -e "${YELLOW}[4/5] Skipping doc tests (quick mode)${NC}"
fi
echo ""

# ==============================================================================
# GATE 5: Clippy + Format (only in full mode)
# ==============================================================================
if $FULL_MODE; then
    echo -e "${YELLOW}[5/5] Running format check and clippy (advisory)...${NC}"
    
    # Check formatting (required)
    if ! cargo fmt --all -- --check 2>&1; then
        echo -e "${RED}✗ GATE 5 FAILED: Code not formatted${NC}"
        echo "Run: cargo fmt --all"
        exit 3
    fi
    echo -e "${GREEN}✓ Format check passed${NC}"
    
    # Run clippy (advisory - warnings don't fail the build)
    echo -e "${YELLOW}Running clippy (advisory)...${NC}"
    cargo clippy --workspace 2>&1 | head -100 || true
    echo -e "${GREEN}✓ Gate 5: Format OK (clippy advisory only)${NC}"
else
    echo -e "${YELLOW}[5/5] Skipping clippy/fmt (use --full to enable)${NC}"
fi

# ==============================================================================
# Summary
# ==============================================================================
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo -e "${BLUE}============================================================${NC}"
echo -e "${GREEN}  ALL GATES PASSED${NC}"
echo -e "${BLUE}============================================================${NC}"
echo ""
echo "Verified:"
echo "  ✓ Namespace isolation (imported bundles filtered by default)"
echo "  ✓ Path safety (directory traversal blocked)"
echo "  ✓ Selfcheck runtime behavior (BLOCKED on real failures)"
echo "  ✓ Support bundle redaction (user paths sanitized)"
echo "  ✓ Core test suite passing"
echo ""
echo -e "Duration: ${DURATION}s"
echo ""

if $QUICK_MODE; then
    echo -e "${YELLOW}Note: Quick mode - some tests skipped. Run without --quick for full verification.${NC}"
fi

if ! $FULL_MODE; then
    echo -e "${YELLOW}Tip: Run with --full to include clippy and format checks.${NC}"
fi

exit 0
