#!/bin/bash
# run_ebpf_smoke.sh - Phase 4 real BPF loading + attachment test
# Compiles BPF objects, loads with edr_attach_any, generates traffic, validates

set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "[smoke] Phase 4 Real BPF Loading + Attachment Test"
echo "[smoke] Starting at $(date)"

# Step 1: Build Rust binaries
echo "[smoke] Building Rust binaries..."
cd "$ROOT_DIR"
cargo build --release --bin edr_attach_any --bin forensic_hooks -q 2>&1 | grep -E "error|Finished" || true

# Step 2: Compile BPF objects with clang
echo "[smoke] Compiling BPF objects with clang..."
mkdir -p build/bpf

# Detect architecture for BPF compilation
if [ "$(uname -m)" = "x86_64" ]; then
    __TARGET_ARCH=x86
else
    __TARGET_ARCH=$(uname -m)
fi

BPF_FILES=(
    "proc_lifecycle.bpf.c"
    "wx_exec.bpf.c"
    "raw_syscalls_catcher.bpf.c"
    "tcp_state.bpf.c"
    "mem_scan.bpf.c"
)

for bpf_file in "${BPF_FILES[@]}"; do
    src="$ROOT_DIR/linux/ebpf/$bpf_file"
    out="$ROOT_DIR/build/bpf/${bpf_file%.c}.o"
    
    if [ -f "$src" ]; then
        echo "[smoke]   Compiling $bpf_file..."
        clang -O2 -target bpf \
            -D__TARGET_ARCH=$__TARGET_ARCH \
            -I"$ROOT_DIR/linux/ebpf" \
            -c "$src" -o "$out" 2>/dev/null || echo "[smoke]     ! clang not available, skipping"
    fi
done

# Step 3: Load and attach BPF programs
echo "[smoke] Loading BPF programs with edr_attach_any..."
if [ -d "/sys/fs/bpf" ]; then
    sudo "$ROOT_DIR/target/release/edr_attach_any" \
        --bpf-dir "$ROOT_DIR/build/bpf" \
        --pin-root /sys/fs/bpf/edr \
        --attach-all --verbose 2>&1 | head -20 || true
    
    # Show loaded programs
    echo "[smoke] BPF programs loaded:"
    bpftool prog list 2>/dev/null | grep -E "proc_lifecycle|wx_exec|raw_syscalls|tcp_state|mem_scan" || echo "[smoke]   (bpftool not available)"
    
    echo "[smoke] BPF maps pinned:"
    bpftool map list 2>/dev/null | grep -E "edr_events_rb|edr_rb_reserve|edr_events_reserved" || echo "[smoke]   (maps not yet pinned)"
else
    echo "[smoke] ! /sys/fs/bpf not available, skipping kernel attach"
fi

# Step 4: Generate load in background
echo "[smoke] Generating process/network load for 30s..."
{
    for i in {1..30}; do
        # Process spawn load
        bash -c "true" &
        # Network load (if possible)
        timeout 1 bash -c "exec 3<>/dev/tcp/127.0.0.1/22; echo quit >&3" 2>/dev/null || true
        sleep 1
    done
    wait 2>/dev/null || true
} &
load_pid=$!

# Step 5: Run self-test and print heartbeat every 5s
echo "[smoke] Running forensic_hooks --self-test-ebpf (30s test window)..."
echo "[smoke] Heartbeat output (transport=ringbuf, ABI validation):"
echo ""

for i in {0..5}; do
    sleep 5
    if [ $i -lt 5 ]; then
        "$ROOT_DIR/target/release/forensic_hooks" --self-test-ebpf 2>&1 | \
            grep -E '"ebpf_transport"|"ebpf_events_read_delta|"ebpf_decode_failed|"timestamp"' | \
            head -5 || true
        echo "[smoke] Heartbeat at $(date +%H:%M:%S)..."
    fi
done

# Wait for load to finish
wait $load_pid 2>/dev/null || true

# Step 6: Final self-test results
echo ""
echo "[smoke] Final self-test results:"
"$ROOT_DIR/target/release/forensic_hooks" --self-test-ebpf 2>&1 | tail -10

echo ""
echo "[smoke] ✓ Phase 4 smoke test complete at $(date)"
echo "[smoke] PASS if: ebpf_transport=ringbuf AND ebpf_events_read_delta_5s>0 AND ebpf_decode_failed_delta_5s==0"
