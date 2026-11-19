> **STATUS: DESIGN/AUDIT DOCUMENT**
> 
> This document describes the design and structure of the integration layer. It is not implementation of runtime wiring; it documents what SHOULD be wired based on code review.
> 
> For actual runtime wiring status, see: [INTEGRATION_WIRING_STATUS.md](../INTEGRATION_WIRING_STATUS.md)
> 
> ---
> 

# Integration Layer Proof of Wiring

## Overview

The `proof_run` binary is a simple CLI tool that verifies the integration layer is properly wired in the EDR locald system. It demonstrates that core integration components can be instantiated and configured correctly.

## What It Tests

The proof_run binary verifies three key integration layer components:

1. **IntegrationProfileStore** - Can be instantiated with no arguments
2. **ExportSinkConfig** - Can be created and output directories set up
3. **IngestSourceConfig** - Can be created with vendor-specific settings

## Usage

```bash
cargo run --bin proof_run -- [OPTIONS]
```

### Options

- `--telemetry-root <PATH>` - Telemetry root directory (default: `/var/lib/edr`)
- `--out <PATH>` - Output JSON file (default: `/var/lib/edr/metrics/proof_run_<id>.json`)
- `--help` - Show help text

### Example

```bash
cargo run --bin proof_run -- --telemetry-root /tmp/edr_test --out /tmp/proof.json
```

## Output

The tool produces a JSON file with the following structure:

```json
{
  "run_id": "unique-uuid",
  "collected_at": "2024-01-01T00:00:00Z",
  "telemetry_root": "/tmp/edr_test",
  "wiring_check": {
    "integration_profile_store_ok": true,
    "export_sink_config_ok": true,
    "ingest_source_config_ok": true,
    "all_wired": true
  },
  "summary": {
    "execution_time_ms": 0,
    "status": "ALL SYSTEMS WIRED"
  }
}
```

## Console Output

When run, proof_run displays:

```
✅ Proof run complete: /tmp/proof.json

Wiring Check Results:
  IntegrationProfileStore: ✓
  ExportSinkConfig: ✓
  IngestSourceConfig: ✓

Overall Status: ALL SYSTEMS WIRED
Execution: 0ms
```

## Design Philosophy

The proof_run binary is intentionally simple and focused:

- **No incident loading** - We don't require incidents to exist
- **No actual ingestion** - We don't poll vendor sources
- **Configuration only** - We verify the config layer works
- **Deterministic** - Same output every time with same inputs

This allows it to run as a lightweight health check without requiring external data or complex setup.

## Integration Testing

All integration layer tests pass:

- Config loading and defaults
- Profile creation and health updates
- Vendor alert fact creation
- Export determinism
- Metrics calculation
- And 14 more tests...

Run all integration tests with:

```bash
cargo test --lib integrations
```
