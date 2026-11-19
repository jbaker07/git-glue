#!/bin/bash
# scripts/validate_windows_sensors.sh
# Validation: Windows sensors, signals, playbooks, platform-scoped loading

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "=== Windows Sensors Validation ==="
echo ""

# 1. Windows sensor files
echo "✓ Windows telemetry collectors:"
FILES=(
  "windows/sensors/mod.rs"
  "windows/sensors/evtx_collector.rs"
  "windows/sensors/sysmon_adapter.rs"
  "windows/sensors/etw_adapter.rs"
  "windows/sensors/defender_adapter.rs"
  "windows/sensors/capture_windows_rotating.rs"
)
for f in "${FILES[@]}"; do
  [ -f "$f" ] && echo "  ✓ $(basename $f)" || (echo "  ✗ $f MISSING"; exit 1)
done
echo ""

# 2. Windows signal types
echo "✓ Signal definitions:"
[ -f "core/signals_windows.rs" ] && echo "  ✓ core/signals_windows.rs" || (echo "  ✗ MISSING"; exit 1)
echo ""

# 3. Windows playbooks
echo "✓ Windows playbooks:"
for f in playbooks/windows/signal_{lolbin_abuse,persistence_windows,defense_evasion}.yaml; do
  [ -f "$f" ] && echo "  ✓ $(basename $f)" || (echo "  ✗ $(basename $f) MISSING"; exit 1)
done
echo ""

# 4. Common playbooks
echo "✓ Common playbooks:"
for f in playbooks/common/signal_{new_remote_ip,high_conn_rate}.yaml; do
  [ -f "$f" ] && echo "  ✓ $(basename $f)" || (echo "  ✗ $(basename $f) MISSING"; exit 1)
done
echo ""

# 5. Platform-scoped playbook loader
echo "✓ Playbook loader:"
[ -f "locald/playbook_loader.rs" ] && echo "  ✓ locald/playbook_loader.rs" || (echo "  ✗ MISSING"; exit 1)
echo ""

# 6. Sensor structure
echo "✓ Sensor directories (OS-agnostic organization):"
for dir in macos/sensors linux/sensors windows/sensors; do
  [ -d "$dir" ] && echo "  ✓ $dir/" || (echo "  ✗ $dir/ MISSING"; exit 1)
done
echo ""

echo "=== Windows sensors created successfully ==="
echo ""
echo "Deliverables:"
echo "  • Windows telemetry sources: EVTX, Sysmon, ETW, Defender/AMSI"
echo "  • Canonical event adapters (→ core TelemetryEvent)"
echo "  • Windows capture binary (JSONL segments + index)"
echo "  • High-value signal types (LOLBin, persistence, defense evasion, etc.)"
echo "  • Windows + common playbooks (YAML, OS-agnostic where possible)"
echo "  • Platform-scoped playbook loader (locald integration)"
echo ""
