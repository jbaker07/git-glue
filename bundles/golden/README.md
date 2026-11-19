# Golden Bundle Library

This directory contains curated "golden" incident bundles used for determinism verification.
These bundles ensure that the recompute pipeline produces stable, reproducible results.

## Purpose

Golden bundles serve two critical functions:

1. **Regression Prevention**: CI runs verify that engine changes don't break determinism
2. **Product Credibility**: Users can verify their installation produces consistent results

## Directory Structure

```
bundles/golden/
├── README.md                           # This file
├── <family>/                           # Hypothesis family grouping
│   └── <bundle_name>/
│       ├── manifest.json               # Bundle metadata + file hashes
│       ├── replay/
│       │   └── report_bundle.json      # Precomputed explanation/report
│       └── recompute/
│           ├── events.jsonl            # Canonical events (sorted by ts, stream_id, segment_id, record_index)
│           ├── config_snapshot.json    # Configuration for deterministic replay
│           └── evidence_excerpts.json  # Optional evidence data
```

## Naming Conventions

- **Family names**: lowercase, underscore-separated (e.g., `credential_access`, `lateral_movement`)
- **Bundle names**: `<family>_<scenario>_<variant>` (e.g., `cred_mimikatz_001`, `persist_schtask_basic`)
- **File names**: exactly as shown in structure (no variations)

## Verification Commands

```bash
# Verify all golden bundles (strict mode)
cargo run -p edr-server --bin golden-cli -- verify --strict bundles/golden

# Verify single bundle
cargo run -p edr-server --bin golden-cli -- verify --strict bundles/golden/credential_access/mimikatz_lsass_001

# Output JSON for CI
cargo run -p edr-server --bin golden-cli -- verify --strict --json results.json bundles/golden

# Makefile helpers
make verify-golden           # Verify all (strict)
make verify-golden-fast      # Verify subset (quick local iteration)
make verify-golden-json      # Verify and output JSON
```

## Verdict Interpretation

| Verdict | Meaning |
|---------|---------|
| `PASS` | Recompute exactly matches replay; all checks passed |
| `PARTIAL` | Recompute matches but with warnings (e.g., version mismatch in best_effort mode) |
| `FAIL` | Checksum mismatch, ordering violation, or determinism failure |

## Adding New Golden Bundles

1. **Generate using fixture generator**:
   ```bash
   # Generate all predefined scenarios
   cargo run -p edr-server --bin golden-cli -- generate bundles/golden
   
   # Or add new scenarios to get_predefined_scenarios() in golden_bundle.rs
   ```

2. **Or manually create** (not recommended):
   - Create directory structure under appropriate family
   - Ensure events.jsonl uses canonical ordering: `ts ASC, stream_id ASC, segment_id ASC, record_index ASC`
   - Compute SHA-256 hashes for manifest.json
   - Export via `/api/export/bundle` with `include_recompute: true`

3. **Verify the new bundle**:
   ```bash
   cargo run -p edr-server --bin golden-cli -- verify bundles/golden/<family>/<bundle_name>
   ```

## Strict vs Best-Effort Mode

- **Strict mode** (`--strict`):
  - Requires component versions in config_snapshot to match current binary
  - Requires playbook fingerprints to match
  - Any mismatch results in `FAIL`

- **Best-effort mode** (default):
  - Runs even with version/fingerprint mismatches
  - Mismatches result in `PARTIAL` verdict with reasons
  - Useful for development and cross-version testing

## Internal vs Shareable Bundles

Golden bundles in this repository are **internal/unredacted** for maximum test fidelity.
They contain synthetic data only (no real PII/hostnames).

For shareable bundles (community sharing, training), always export with `redact: true`.

## Coverage Requirements

Minimum golden bundle coverage:

| Family | Scenarios | Status |
|--------|-----------|--------|
| credential_access | mimikatz, cmdline_pattern, lsass_dump | ✓ |
| persistence | scheduled_task, registry_run, service_create | ✓ |
| lateral_movement | psexec, wmi_remote, ssh_pivot | ✓ |
| defense_evasion | process_hollow, dll_sideload, timestomp | ✓ |
| exfiltration | chunked_https, dns_tunnel | ✓ |
| process_injection | dll_inject, apc_inject | ✓ |
| dns_anomaly | tunnel_pattern, dga_pattern | ✓ |
| privilege_escalation | token_impersonate, uac_bypass | ✓ |
| edge_cases | late_arrival, throttling_summary, visibility_degraded | ✓ |

## CI Integration

The CI job (`.github/workflows/golden-verify.yml`) runs on every push:

1. Checks out repository
2. Runs `make verify-golden` (strict mode)
3. Fails build if any bundle fails
4. Uploads `golden-verify-results.json` as artifact

## Troubleshooting

### "Checksum mismatch"
- Bundle was modified after generation
- Regenerate: `cargo run -p edr-server --bin golden-cli -- generate bundles/golden`

### "Ordering violation"
- events.jsonl not sorted canonically
- Check: `ts`, then `stream_id`, then `segment_id`, then `record_index`

### "Version mismatch" in strict mode
- config_snapshot.component_versions.engine differs from current binary
- Either update bundle or use best_effort mode

### "Timeline mismatch"
- Determinism bug in the pipeline
- This should fail CI - investigate before merging
