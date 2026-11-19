# Pre-Push Checklist

## Before Every Push

### 1. Build Verification
```bash
# Full release build - MUST succeed
cargo build --release --bins

# Quick check - all binaries exist
ls target/release/{edr-server,edr-locald,proof_run,capture_*}
```

### 2. Test Suite
```bash
# Unit tests
cargo test --workspace

# If server is running, smoke test
./scripts/smoke_stack.sh
```

### 3. Proof Artifact
```bash
# Generate proof artifact
EDR_TELEMETRY_ROOT=./test_telemetry ./target/release/proof_run

# Verify JSON was written
cat test_telemetry/proof-*.json | jq .summary
```

### 4. Git Hygiene
```bash
# Check for untracked files that should be ignored
git status

# Ensure no binaries are staged
git diff --cached --name-only | grep -E '\.(exe|dll|so|dylib)$' && echo "ERROR: Binary staged!"

# Ensure no secrets are staged
git diff --cached | grep -iE '(password|secret|token|api.?key)' && echo "WARNING: Possible secret!"
```

### 5. Warning Count (informational)
```bash
cargo build --release 2>&1 | grep -c "^warning:"
# Expect ~50 warnings - see docs/DEV_NOTES.md
```

---

## For Release Tags

### Additional Steps

1. **Update version** in all `Cargo.toml` files
2. **Update CHANGELOG.md** (if exists)
3. **Run on all platforms** or verify CI passed:
   - macOS (native)
   - Linux VM
   - Windows VM

### Version Bump Script
```bash
# Example: bump to 0.2.0
find . -name Cargo.toml -exec sed -i '' 's/version = "0.1.0"/version = "0.2.0"/g' {} \;
cargo build --release  # Verify Cargo.lock updates
```

---

## Quick Pre-Push (Copy/Paste)

```bash
# All-in-one check (macOS/Linux)
cargo build --release --bins && \
cargo test --workspace && \
EDR_TELEMETRY_ROOT=./test_telemetry ./target/release/proof_run && \
echo "✓ Ready to push"
```

```powershell
# All-in-one check (Windows PowerShell)
cargo build --release --bins; `
if ($?) { cargo test --workspace }; `
if ($?) { $env:EDR_TELEMETRY_ROOT=".\test_telemetry"; .\target\release\proof_run.exe }; `
if ($?) { Write-Host "✓ Ready to push" -ForegroundColor Green }
```

---

## Files That Should NOT Be Committed

- `target/` - Build artifacts
- `test_telemetry/` - Runtime telemetry
- `*.db` - SQLite databases
- `segments/` - Telemetry segments
- `exports/` - Export outputs
- `.env` - Environment files (if any)

These are all in `.gitignore`.
