#!/bin/bash
# Stop macOS EDR pipeline

set -e

PORT=8000
WIPE_TELEMETRY=false

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        --port)
            PORT="$2"
            shift 2
            ;;
        --wipe)
            WIPE_TELEMETRY=true
            shift
            ;;
        *)
            echo "Usage: $0 [--port PORT] [--wipe]"
            exit 1
            ;;
    esac
done

echo "[stop_macos] Stopping EDR pipeline (port $PORT)..."

# Kill processes
for proc in capture_macos_rotating edr_locald ui_server; do
    pids=$(pgrep -f "$proc" || true)
    if [ -n "$pids" ]; then
        echo "[stop_macos] Killing $proc..."
        echo "$pids" | xargs kill -9 2>/dev/null || true
    fi
done

# Close port
lsof -nP -iTCP:$PORT -sTCP:LISTEN | awk 'NR>1 {print $2}' | xargs kill -9 2>/dev/null || true

echo "[stop_macos] Processes stopped"

# Optional wipe
if [ "$WIPE_TELEMETRY" = true ]; then
    read -p "Wipe telemetry_output? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ -d "edr-agent/telemetry_output" ]; then
            rm -rf edr-agent/telemetry_output
            mkdir -p edr-agent/telemetry_output/segments
            echo "[stop_macos] Telemetry wiped"
        fi
    fi
fi

echo "[stop_macos] Clean shutdown complete"
