#!/bin/bash
# test_ebpf_e2e.sh - End-to-end validation of eBPF setup on Linux
# Usage: sudo bash test_ebpf_e2e.sh

set -e

if [ "$EUID" -ne 0 ]; then
    echo "ERROR: This script requires root (sudo)"
    exit 1
fi

EBPF_DIR="${EBPF_DIR:-linux/ebpf}"
OUTPUT_DIR="${OUTPUT_DIR:-target/debug/bpf}"
PIN_ROOT="${PIN_ROOT:-/sys/fs/bpf/edr}"

echo "================================"
echo "  eBPF End-to-End Test Suite"
echo "================================"
echo ""

# Step 1: Check kernel requirements
echo "[1/5] Checking kernel capabilities..."
if ! grep -q "CONFIG_BPF=y" /boot/config-$(uname -r) 2>/dev/null; then
    echo "  ✗ BPF not enabled in kernel"
    exit 1
fi
echo "  ✓ Kernel BPF support detected"

# Step 2: Check eBPF objects exist
echo "[2/5] Checking compiled .bpf.o files..."
if [ ! -d "$OUTPUT_DIR" ]; then
    echo "  ✗ Output directory not found: $OUTPUT_DIR"
    echo "    Run: bash build_ebpf.sh"
    exit 1
fi

OBJ_COUNT=$(ls -1 "$OUTPUT_DIR"/*.bpf.o 2>/dev/null | wc -l)
if [ "$OBJ_COUNT" -lt 10 ]; then
    echo "  ✗ Found only $OBJ_COUNT .bpf.o files (expected ≥10)"
    exit 1
fi
echo "  ✓ Found $OBJ_COUNT .bpf.o objects"

# Step 3: Clean up old pinned maps
echo "[3/5] Cleaning up old pinned maps..."
if [ -d "$PIN_ROOT" ]; then
    rm -rf "$PIN_ROOT" 2>/dev/null || true
fi
echo "  ✓ Pin directory cleared"

# Step 4: Load eBPF programs
echo "[4/5] Loading eBPF programs with edr_attach_any..."
if ! command -v edr_attach_any &> /dev/null; then
    if [ ! -f "target/debug/edr_attach_any" ]; then
        echo "  ✗ edr_attach_any binary not found"
        echo "    Run: cargo build --bins"
        exit 1
    fi
    LOADER="target/debug/edr_attach_any"
else
    LOADER="edr_attach_any"
fi

if ! $LOADER --bpf-dir "$OUTPUT_DIR" --verbose --ttl-seconds 5; then
    echo "  ✗ Failed to load eBPF programs"
    exit 1
fi
echo "  ✓ eBPF programs loaded"

# Step 5: Test ringbuf reader
echo "[5/5] Testing ringbuf reader with capture_linux_rotating..."
if [ ! -f "target/debug/capture_linux_rotating" ]; then
    echo "  ✗ capture_linux_rotating binary not found"
    exit 1
fi

if timeout 40s target/debug/capture_linux_rotating --pin-root "$PIN_ROOT" --self-test-ebpf; then
    echo "  ✓ Ringbuf reader validation PASSED"
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo "  ✗ Test timeout (no events detected)"
    else
        echo "  ✗ Ringbuf reader test failed (exit: $EXIT_CODE)"
    fi
    exit 1
fi

echo ""
echo "================================"
echo "  ✓ ALL TESTS PASSED"
echo "================================"
echo ""
echo "Next: Run continuous capture with:"
echo "  sudo target/debug/capture_linux_rotating --output /tmp/capture --pin-root $PIN_ROOT"
