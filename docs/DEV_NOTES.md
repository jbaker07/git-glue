# Developer Notes

## Known Warnings

The codebase has approximately 50 warnings, primarily:

| Category | Count | Notes |
|----------|-------|-------|
| Unused imports | ~30 | Safe to ignore; run `cargo fix --allow-dirty` to auto-remove |
| Unused fields | ~10 | Intentional - reserved for future OS-specific implementations |
| Dead code | ~8 | Stubs and placeholders awaiting platform implementation |
| Unused variables | ~2 | Pattern match bindings in event handlers |

### Suppression Strategy

We do NOT use `#[allow(dead_code)]` broadly. Warnings serve as a TODO list for:
1. **ES (Endpoint Security)** - macOS-only, stubs on other platforms
2. **eBPF sensors** - Linux-only, stubs on other platforms
3. **ETW sensors** - Windows-only, stubs on other platforms

### Quick Cleanup

```bash
# Auto-fix obvious unused imports
cargo fix --allow-dirty --lib -p edr-locald
cargo fix --allow-dirty --lib -p edr-core

# Don't auto-fix bins - they may have intentional conditional compilation
```

## Capability Gating

ES (Endpoint Security) stubs are gated by `ESCapabilityStatus`:

```rust
// In crates/agent-macos/src/sensors/es/mod.rs
pub fn should_emit_es_facts() -> bool {
    match es_capability_status() {
        ESCapabilityStatus::Full => true,
        ESCapabilityStatus::Limited => true,
        ESCapabilityStatus::Unimplemented => false,
    }
}
```

**Current status**: `Unimplemented` - ES stubs produce zero facts.

## Build Variants

| OS | Default Agent | ES Support | eBPF Support | ETW Support |
|----|---------------|------------|--------------|-------------|
| macOS | capture_macos_rotating | Stubbed | N/A | N/A |
| Linux | capture_linux_rotating | N/A | Stubbed | N/A |
| Windows | capture_windows_rotating | N/A | N/A | Stubbed |

## Port Configuration

| Component | Default | Env Var | CLI Flag |
|-----------|---------|---------|----------|
| edr-server | 3000 | `EDR_SERVER_PORT` | `--port` |
| API URL | http://127.0.0.1:3000 | `EDR_API_URL` | N/A |

## Telemetry Root

| OS | Default Path | Env Override |
|----|--------------|--------------|
| macOS | `./test_telemetry` | `EDR_TELEMETRY_ROOT` |
| Linux | `/var/lib/edr` | `EDR_TELEMETRY_ROOT` |
| Windows | `C:\ProgramData\edr` | `EDR_TELEMETRY_ROOT` |

## Testing

```bash
# Unit tests
cargo test --workspace

# Integration (requires server running)
./scripts/smoke_stack.sh

# Proof artifact
./target/release/proof_run
```

## Common Issues

### "curl: (7) Failed to connect"
Server isn't running. Start with `./target/release/edr-server`.

### "Permission denied" on Linux
Telemetry root `/var/lib/edr` needs sudo or change to `EDR_TELEMETRY_ROOT=$HOME/.edr`.

### Windows path issues
Use forward slashes in Git Bash, backslashes in PowerShell.

## Architecture Decisions

1. **Single workspace** - All crates in one repo for atomic versioning
2. **Platform agents as separate binaries** - Avoids conditional compilation hell
3. **Stub-first approach** - All sensors compile on all platforms, emit nothing when unavailable
4. **JSON-first** - All telemetry and config is JSON for tooling interop
