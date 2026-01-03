#!/bin/bash
# run_stack_macos.sh - Start EDR stack on macOS
# Requires: cargo, sudo access for capture
#
# Usage:
#   ./scripts/run_stack_macos.sh              # Build and run
#   ./scripts/run_stack_macos.sh --no-build   # Run pre-built binaries

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Configuration
export EDR_TELEMETRY_ROOT="${EDR_TELEMETRY_ROOT:-$PROJECT_ROOT/telemetry_data}"
WAIT_SECONDS=60
BUILD=true

# Parse args
for arg in "$@"; do
    case $arg in
        --no-build) BUILD=false ;;
        --help|-h)
            echo "Usage: $0 [--no-build]"
            echo "  --no-build  Skip cargo build, use pre-built binaries"
            exit 0
            ;;
    esac
done

echo "=========================================="
echo " EDR Stack - macOS"
echo "=========================================="
echo "EDR_TELEMETRY_ROOT: $EDR_TELEMETRY_ROOT"
echo ""

# Create directories
mkdir -p "$EDR_TELEMETRY_ROOT/segments"
mkdir -p "$EDR_TELEMETRY_ROOT/incidents/default"
mkdir -p "$EDR_TELEMETRY_ROOT/exports/default"
mkdir -p "$EDR_TELEMETRY_ROOT/metrics"

# Create empty index.json if not exists
if [ ! -f "$EDR_TELEMETRY_ROOT/segments/index.json" ]; then
    echo '{"segments":[]}' > "$EDR_TELEMETRY_ROOT/segments/index.json"
fi

# Build if requested
if [ "$BUILD" = true ]; then
    echo "[1/5] Building release binaries..."
    cargo build --release -p agent-macos -p edr-locald -p edr-server
fi

# Binary paths
CAPTURE_BIN="$PROJECT_ROOT/target/release/capture_macos_rotating"
LOCALD_BIN="$PROJECT_ROOT/target/release/edr-locald"
SERVER_BIN="$PROJECT_ROOT/target/release/edr-server"

# Verify binaries exist
for bin in "$CAPTURE_BIN" "$LOCALD_BIN" "$SERVER_BIN"; do
    if [ ! -f "$bin" ]; then
        echo "ERROR: Binary not found: $bin"
        echo "Run without --no-build to compile"
        exit 1
    fi
done

# PID tracking for cleanup
PIDS=()

cleanup() {
    echo ""
    echo "[SHUTDOWN] Stopping all processes..."
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done
    wait 2>/dev/null
    echo "[SHUTDOWN] Complete"
}

trap cleanup EXIT INT TERM

echo "[2/5] Starting capture_macos_rotating (requires sudo)..."
sudo -E "$CAPTURE_BIN" > "$EDR_TELEMETRY_ROOT/capture.log" 2>&1 &
PIDS+=($!)
echo "  PID: ${PIDS[-1]}"

sleep 2

echo "[3/5] Starting edr-locald..."
"$LOCALD_BIN" > "$EDR_TELEMETRY_ROOT/locald.log" 2>&1 &
PIDS+=($!)
echo "  PID: ${PIDS[-1]}"

sleep 1

echo "[4/5] Starting edr-server..."
"$SERVER_BIN" > "$EDR_TELEMETRY_ROOT/server.log" 2>&1 &
PIDS+=($!)
echo "  PID: ${PIDS[-1]}"

echo ""
echo "=========================================="
echo " Stack Running"
echo "=========================================="
echo "UI URL:        http://localhost:3000"
echo "API Health:    http://localhost:3000/api/health"
echo "Incidents:     http://localhost:3000/api/incidents"
echo ""
echo "Logs:"
echo "  Capture:  tail -f $EDR_TELEMETRY_ROOT/capture.log"
echo "  Locald:   tail -f $EDR_TELEMETRY_ROOT/locald.log"
echo "  Server:   tail -f $EDR_TELEMETRY_ROOT/server.log"
echo ""

echo "[5/5] Waiting $WAIT_SECONDS seconds for events to accumulate..."
sleep $WAIT_SECONDS

# Run proof_run
echo ""
echo "=========================================="
echo " Running proof_run"
echo "=========================================="
PROOF_RUN_BIN="$PROJECT_ROOT/target/release/proof_run"
if [ -f "$PROOF_RUN_BIN" ]; then
    "$PROOF_RUN_BIN" || echo "[proof_run] Non-zero exit (may be expected)"
    echo ""
    echo "Proof artifacts:"
    find "$EDR_TELEMETRY_ROOT" -name "proof_run*.json" -o -name "incidents*.jsonl" 2>/dev/null | head -10
else
    echo "[WARN] proof_run binary not found. Build with: cargo build --release -p edr-locald"
fi

echo ""
echo "=========================================="
echo " Summary"
echo "=========================================="
echo "Segments written: $(ls -1 "$EDR_TELEMETRY_ROOT/segments"/*.jsonl 2>/dev/null | wc -l | tr -d ' ')"
echo "Incidents:        $(ls -1 "$EDR_TELEMETRY_ROOT/incidents/default"/*.json 2>/dev/null | wc -l | tr -d ' ')"
echo ""
echo "Press Ctrl-C to stop the stack..."
echo ""

# Wait for interrupt
wait
