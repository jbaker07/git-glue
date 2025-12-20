#!/bin/bash
# scripts/validate_windows_sensors.sh
# Short validation: Windows sensors, signal definitions, playbooks, platform-scoped loading

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "=== Windows Sensors Validation ==="
echo ""

# 1. Check Windows sensors files exist
echo "✓ Windows telemetry sources:"
for f in windows/sensors/{mod,evtx_collector,sysmon_adapter,etw_adapter,defender_adapter,capture_windows_rotating}.rs; do
  if [ -f "$f" ]; then echo "  ✓ $f"; else echo "  ✗ $f MISSING"; exit 1; fi
done
echo ""

# 2. Check Windows signal definitions
echo "✓ Signal definitions:"
if [ -f "core/signals_windows.rs" ]; then
  echo "  ✓ core/signals_windows.rs"
else
  echo "  ✗ core/signals_windows.rs MISSING"
  exit 1
fi
echo ""

# 3. Check Windows playbooks
echo "✓ Windows playbooks:"
for f in playbooks/windows/{signal_lolbin_abuse,signal_persistence_windows,signal_defense_evasion}.yaml; do
  if [ -f "$f" ]; then echo "  ✓ $(basename $f)"; else echo "  ✗ $(basename $f) MISSING"; exit 1; fi
done
echo ""

# 4. Check common playbooks
echo "✓ Common playbooks:"
for f in playbooks/common/{signal_new_remote_ip,signal_high_conn_rate}.yaml; do
  if [ -f "$f" ]; then echo "  ✓ $(basename $f)"; else echo "  ✗ $(basename $f) MISSING"; exit 1; fi
done
echo ""

# 5. Check playbook loader
echo "✓ Platform-scoped playbook loader:"
if [ -f "locald/playbook_loader.rs" ]; then
  echo "  ✓ locald/playbook_loader.rs"
else
  echo "  ✗ locald/playbook_loader.rs MISSING"
  exit 1
fi
echo ""

# 6. Cargo check (core only, Windows capture is cfg(windows))
echo "✓ Cargo check (core + locald):"
if ! cargo check --lib --manifest-path core/lib.rs 2>&1 | grep -q "error"; then
  echo "  ✓ core compiles"
else
  echo "  ✗ core has errors"
  exit 1
fi

echo ""
echo "=== All validations passed ==="
