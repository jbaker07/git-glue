#!/bin/bash
# build_ebpf.sh - Compile eBPF programs and prepare for loading
# Run this on Linux with clang and kernel headers installed

set -e

EBPF_DIR="${1:-linux/ebpf}"
OUTPUT_DIR="${2:-target/debug/bpf}"

echo "[build_ebpf] Compiling eBPF programs"
echo "[build_ebpf] Input: $EBPF_DIR"
echo "[build_ebpf] Output: $OUTPUT_DIR"

if [ ! -d "$EBPF_DIR" ]; then
    echo "[build_ebpf] ERROR: eBPF directory not found: $EBPF_DIR"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

# Check for clang
if ! command -v clang &> /dev/null; then
    echo "[build_ebpf] ERROR: clang not found. Install: apt install clang llvm-dev"
    exit 1
fi

# Check for vmlinux.h
if [ ! -f "$EBPF_DIR/include/vmlinux.h" ]; then
    echo "[build_ebpf] WARNING: vmlinux.h missing. Using bpftool to generate..."
    if command -v bpftool &> /dev/null; then
        bpftool btf dump file /sys/kernel/btf/vmlinux format c > "$EBPF_DIR/include/vmlinux.h"
        echo "[build_ebpf] Generated vmlinux.h"
    else
        echo "[build_ebpf] ERROR: bpftool not found. Cannot generate vmlinux.h"
        echo "[build_ebpf] Install: apt install linux-headers-\$(uname -r) linux-tools-\$(uname -r)"
        exit 1
    fi
fi

# Build eBPF programs
cd "$EBPF_DIR"
make clean
make

# Copy .bpf.o files to output directory
for obj in *.bpf.o; do
    if [ -f "$obj" ]; then
        cp "$obj" "../../$OUTPUT_DIR/"
        echo "[build_ebpf] ✓ $obj"
    fi
done

echo "[build_ebpf] ✓ Complete. Objects in $OUTPUT_DIR"
echo "[build_ebpf] Next: sudo target/debug/edr_attach_any --bpf-dir $OUTPUT_DIR --verbose"
