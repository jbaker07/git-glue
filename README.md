# forensic-hooks: Behavioral EDR Core

A macOS-native behavioral detection engine that converts kernel-level events into high-fidelity security incidents through process-centric signal aggregation, playbook-driven correlation, and lightweight machine-learning-free scoring.

## What Runs

**Tier-0 (Shipping):** Always-compile requirement
- `forensic_hooks` — Main EDR detection server (HTTP API, incident streaming, signal aggregation)

**Tier-1 (Optional):** May have compile errors; supplemental features
- `fingerprint_gen` — Offline fingerprint generation
- `edr_wrap` — Wrapper/agent harness
- `capture_macos_rotating` — Kernel event capture (WIP: unresolved imports)
- `edr_locald` — Local EDR daemon (WIP)
- `ui_server` — Web UI and REST API (WIP)

## Quickstart

```bash
# Build library and Tier-0
cargo check --lib
cargo check --bin forensic_hooks

# Run Tier-0 gate validation
bash scripts/tier0_gates.sh

# Run Tier-1 gate validation (optional)
bash scripts/tier1_gates.sh

# Build a release binary
cargo build --release --bin forensic_hooks
```

## Detection Architecture

Events flow: **BSM/ESF kernel events** → **Fact ingest** → **Signal aggregation (rolling windows)** → **Playbook correlation** → **Final incident alerts**

- **Events**: Process lifecycle (fork, exec, exit), file operations, network, system calls
- **Facts**: Normalized, tagged events with semantic context (parent, arguments, user)
- **Signals**: Time-windowed summaries (e.g., "rapid process spawn", "binary wrote to /tmp")
- **Playbooks**: Stateful rule engines that correlate signals (e.g., "fork bomb → escalation attempt")
- **Incidents**: Final scored alerts with evidence trails

Detection is **architecture-driven**: no external ML models required; confidence scoring is threshold-based and configurable.

## Repo Map

- `src/` — Core EDR library
  - `src/bin/` — Public entry points (Tier-0/Tier-1 binaries)
  - `src/detect/` — Detection rules and heuristics
  - `src/modules/` — Behavioral sensors (process, file, network)
  - `src/macos_detect/` — macOS BSM/ESF parsing and state machines
  - `src/trust/` — Trust/confidence scoring algorithms
  - `src/ebpf/` — eBPF programs (Linux optional)
  - `src/pb_adapter/` — Playbook engine bridge
  - `src/services/` — HTTP API, SSE streaming, healthz
  - `src/forensic/` — Forensic export, evidence bundling
  - `src/modes/` — Operational modes (demo, YARA, policy-driven)
  - `src/gnn/` — Graph neural network integration (feature-gated, OFF by default)

- `tools/` — Offline utilities
  - `tools/offline_utilities/` — Playbook validation, policy loader, semantic enrichment
  - `scripts/` — CI/validation gates, dead-code detectors

- `attic/` — Legacy code (not shipped)
  - `attic/rust_deadcode/` — Unreachable Rust modules (safe to clean up)
  - `attic/legacy_python/` — Old Python analyzers
  - `attic/trust_legacy/` — Previous trust model versions
  - `attic/experiments/` — Proof-of-concept code

## Feature Flags

All feature flags are **OFF by default** (standard `cargo build` does not enable them):

- `gnn` — Graph neural network vector logging and correlation (requires external GNN service)
- `forensic` — Detailed forensic bundle export (adds disk overhead)
- `advanced_scoring` — YARA/Elastic/ECS-based scoring (requires external feeds)

To enable a feature:
```bash
cargo build --features gnn,forensic,advanced_scoring
```

## Building for Production

1. Validate gates:
   ```bash
   bash scripts/tier0_gates.sh
   ```

2. Build Tier-0 release:
   ```bash
   cargo build --release --bin forensic_hooks --bin capture_macos_rotating --bin edr_locald --bin ui_server
   ```

3. Deploy: Copy binaries to target system; point to policy/ontology JSON files.

## Development

- Tier-0 must always compile: any breaking change requires a feature gate or module move to `tools/` or `attic/`
- Tests: Run `cargo test --lib` (unit tests only; integration tests in separate harness)
- Code organization: New detection logic goes under `src/detect/` or `src/modules/`

## What This Is NOT

- Not a full SIEM: generates incidents, not a log ingestion system
- Not a threat feed: no built-in IOC or threat intelligence
- Not cloud-native: designed for on-box or local network deployment
- Not general-purpose: optimized for macOS BSM/ESF; Linux eBPF is supplemental

## License

[Specified in LICENSE file]
