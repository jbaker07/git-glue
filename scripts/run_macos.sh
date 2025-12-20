#!/bin/bash
# run_macos.sh: Orchestrate the 3-binary macOS EDR pipeline
#
# Binary 1: capture_macos_rotating (root privilege)
# Binary 2: edr_locald (user privilege)
# Binary 3: ui_server (user privilege, HTTP API)
#
# Usage: ./scripts/run_macos.sh [--port 8000] [--preflight]

set -e

# ===== Configuration =====
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRATE_DIR="$WORKSPACE_ROOT/git-glue"

# TELEMETRY_ROOT: Can be overridden with EDR_TELEMETRY_ROOT env var
# Production: $WORKSPACE_ROOT/EDR-platform/edr-agent/telemetry_output
TELEMETRY_ROOT="${EDR_TELEMETRY_ROOT:-$WORKSPACE_ROOT/EDR-platform/edr-agent/telemetry_output}"

# Default port
PORT=8000

# ===== Colors =====
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# ===== Logging Functions =====
log_info() {
    echo -e "${BLUE}[*]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[+]${NC} $1"
}

log_error() {
    echo -e "${RED}[!]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[~]${NC} $1"
}

# ===== Preflight Checks =====
run_preflight() {
    log_info "Running preflight checks..."
    echo ""
    
    # Check repo structure
    log_info "Checking repo structure..."
    [[ -d "$REPO_ROOT/git-glue" ]] || { log_error "git-glue directory not found"; exit 1; }
    [[ -f "$CRATE_DIR/Cargo.toml" ]] || { log_error "Cargo.toml not found"; exit 1; }
    [[ -d "$CRATE_DIR/src/bin" ]] || { log_error "src/bin not found"; exit 1; }
    log_success "Repository structure valid"
    echo ""
    
    # Check binaries are defined
    log_info "Checking binaries in Cargo.toml..."
    grep -q 'name = "capture_macos_rotating"' "$CRATE_DIR/Cargo.toml" || { log_error "capture_macos_rotating binary not defined"; exit 1; }
    grep -q 'name = "edr_locald"' "$CRATE_DIR/Cargo.toml" || { log_error "edr_locald binary not defined"; exit 1; }
    grep -q 'name = "ui_server"' "$CRATE_DIR/Cargo.toml" || { log_error "ui_server binary not defined"; exit 1; }
    log_success "All 3 binaries defined"
    echo ""
    
    # Check if binaries are built
    log_info "Checking if binaries are built..."
    CAPTURE_BIN="$CRATE_DIR/target/release/capture_macos_rotating"
    LOCALD_BIN="$CRATE_DIR/target/release/edr_locald"
    UI_BIN="$CRATE_DIR/target/release/ui_server"
    
    if [[ ! -f "$CAPTURE_BIN" ]] || [[ ! -f "$LOCALD_BIN" ]] || [[ ! -f "$UI_BIN" ]]; then
        log_warn "Binaries not found. Building..."
        cd "$CRATE_DIR"
        cargo build --release --bins 2>&1 | grep -E "(Compiling|Finished|error)" || true
        cd "$REPO_ROOT"
    fi
    
    [[ -f "$CAPTURE_BIN" ]] || { log_error "capture_macos_rotating binary not found at $CAPTURE_BIN"; exit 1; }
    [[ -f "$LOCALD_BIN" ]] || { log_error "edr_locald binary not found at $LOCALD_BIN"; exit 1; }
    [[ -f "$UI_BIN" ]] || { log_error "ui_server binary not found at $UI_BIN"; exit 1; }
    log_success "All binaries present"
    echo ""
    
    # Check TELEMETRY_ROOT
    log_info "TELEMETRY_ROOT: $TELEMETRY_ROOT"
    log_success "Preflight passed"
    echo ""
}

# ===== Argument Parsing =====
PREFLIGHT=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --port)
            if [[ $# -lt 2 ]]; then
                log_error "--port requires a value"
                exit 1
            fi
            PORT="$2"
            shift 2
            ;;
        --preflight)
            PREFLIGHT=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run preflight if requested
if [[ "$PREFLIGHT" == true ]]; then
    run_preflight
    exit 0
fi

# ===== Setup =====
log_info "Setting up EDR pipeline..."
log_info "WORKSPACE_ROOT: $WORKSPACE_ROOT"
log_info "TELEMETRY_ROOT: $TELEMETRY_ROOT"
log_info "PORT: $PORT"
echo ""

# Create directories
log_info "Creating directories..."
mkdir -p "$TELEMETRY_ROOT/segments"
mkdir -p "$TELEMETRY_ROOT/logs"
log_success "Directories ready"
echo ""

# Ensure binaries are built
log_info "Building binaries..."
cd "$CRATE_DIR"
cargo build --release --bins 2>&1 | tail -1 | grep -q "Finished" && log_success "Build successful" || {
    log_error "Build failed"
    exit 1
}
cd "$WORKSPACE_ROOT"
echo ""

# ===== Start Binaries =====
CAPTURE_BIN="$CRATE_DIR/target/release/capture_macos_rotating"
LOCALD_BIN="$CRATE_DIR/target/release/edr_locald"
UI_BIN="$CRATE_DIR/target/release/ui_server"

log_info "Starting EDR pipeline..."
echo ""

# Cleanup function
cleanup() {
    log_warn "Shutting down..."
    kill $CAPTURE_PID 2>/dev/null || true
    kill $LOCALD_PID 2>/dev/null || true
    kill $UI_PID 2>/dev/null || true
    wait 2>/dev/null || true
    log_success "Shutdown complete"
}

trap cleanup SIGINT SIGTERM

# Start capture (requires root, run with sudo)
log_info "Starting capture_macos_rotating (requires sudo)..."
if sudo -n true 2>/dev/null; then
    # sudo cached password
    sudo env EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT" "$CAPTURE_BIN" > "$TELEMETRY_ROOT/logs/capture.log" 2>&1 &
    CAPTURE_PID=$!
    log_success "capture_macos_rotating (PID $CAPTURE_PID)"
else
    log_warn "sudo access not available; skipping capture (it requires root)"
    log_info "To run with capture, use: sudo -v before running this script"
    CAPTURE_PID=""
fi

# Start edr_locald (no privilege required)
log_info "Starting edr_locald..."
env EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT" "$LOCALD_BIN" > "$TELEMETRY_ROOT/logs/locald.log" 2>&1 &
LOCALD_PID=$!
log_success "edr_locald (PID $LOCALD_PID)"

# Start ui_server (no privilege required)
log_info "Starting ui_server on port $PORT..."
env EDR_TELEMETRY_ROOT="$TELEMETRY_ROOT" "$UI_BIN" --port "$PORT" > "$TELEMETRY_ROOT/logs/ui_server.log" 2>&1 &
UI_PID=$!
log_success "ui_server (PID $UI_PID)"

echo ""
log_success "EDR Pipeline Running"
log_info "Logs: $TELEMETRY_ROOT/logs/"
log_info "API: http://127.0.0.1:$PORT/api/health"
echo ""

# Wait for all processes
wait
