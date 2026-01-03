# REPRODUCIBLE VERIFICATION COMMANDS

Run these commands to independently verify the audit findings.

## PREREQUISITE
```bash
cd /Users/jermainebaker/git-glue
```

---

## 1. VERIFY TESTS PASS ✅

```bash
# This PASSES
cargo test --lib 2>&1 | tail -20
```

**Expected output:**
```
test result: ok. 201 passed; 0 failed; 0 ignored; 0 measured
```

**Evidence:** Unit tests are working correctly.

---

## 2. VERIFY EXPORT NOT CALLED AT RUNTIME ❌

```bash
# Count: IncidentExporter::new() calls OUTSIDE of tests
grep -r "IncidentExporter::new" src --include="*.rs" | grep -v test | wc -l
```

**Expected output:**
```
0
```

**Evidence:** IncidentExporter is never instantiated in production code.

---

## 3. VERIFY INGEST NOT CALLED AT RUNTIME ❌

```bash
# Count: .poll() calls OUTSIDE of tests
grep -r "\.poll()" src --include="*.rs" | grep -v test | wc -l
```

**Expected output:**
```
0
```

**Evidence:** Ingest poll loop is never executed at runtime.

---

## 4. VERIFY PROFILE STORE NOT UPDATED AT RUNTIME ❌

```bash
# Count: Profile store update method calls OUTSIDE of tests
grep -r "\.upsert_profile\|\.record_sample\|\.update_health" src --include="*.rs" | grep -v test | wc -l
```

**Expected output:**
```
0
```

**Evidence:** Profile store statistics are never recorded.

---

## 5. VERIFY UI_SERVER MISSING ❌

```bash
# Check if ui_server.rs exists
ls src/bin/ui_server.rs 2>&1
```

**Expected output:**
```
ls: src/bin/ui_server.rs: No such file or directory
```

**Evidence:** UI server binary doesn't exist.

---

## 6. VERIFY proof_run BROKEN ❌

```bash
# Try to build proof_run
cargo build --release --bin proof_run 2>&1 | grep -E "error|Finished"
```

**Expected output:**
```
error: path `../core` does not exist
```

**Evidence:** proof_run can't build due to missing edr-core.

---

## 7. VERIFY AUDIT DOCS NOT COMMITTED ❌

```bash
# Check if audit docs are in git
git ls-files docs/ | grep -i "audit\|smoke"
```

**Expected output:**
```
(empty)
```

**Evidence:** Audit documents exist on disk but aren't tracked by git.

---

## 8. VERIFY SMOKE SCRIPT MISSING ❌

```bash
# Check if smoke test script exists
ls scripts/smoke_integrations.sh 2>&1
```

**Expected output:**
```
ls: scripts/smoke_integrations.sh: No such file or directory
```

**Evidence:** Smoke test script doesn't exist.

---

## 9. VERIFY MAIN.RS IS STUB ❌

```bash
# Show that main.rs has no runtime logic
cat src/main.rs
```

**Expected output:**
```
//! EDR Local Daemon - use as library or run with proper config

fn main() {
    println!("EDR Local Daemon - use as library or run with proper config");
}
```

**Evidence:** Main entry point is just a help message.

---

## 10. VERIFY EXPORT STRUCTURE EXISTS ✅

```bash
# Show export.rs exists and has methods
grep "pub fn" src/integrations/export.rs | head -10
```

**Expected output:**
```
pub fn new(config: ExportSinkConfig) -> Result<Self, String> {
pub fn export_batch(&mut self, incidents: &[&Incident], namespace: Option<&str>) -> Result<ExportResult, String> {
pub fn export_fresh(&mut self, incidents: &[&Incident], namespace: Option<&str>) -> Result<ExportResult, String> {
```

**Evidence:** IncidentExporter is fully implemented, just not used.

---

## SUMMARY COMMAND

Run all checks in one script:

```bash
#!/bin/bash
set -e

echo "=== VERIFICATION AUDIT ==="
echo ""

echo "1. Tests passing?"
cargo test --lib 2>&1 | grep "test result"

echo ""
echo "2. Export called at runtime?"
EXPORT_CALLS=$(grep -r "IncidentExporter::new" src --include="*.rs" | grep -v test | wc -l)
if [ "$EXPORT_CALLS" -eq 0 ]; then echo "❌ NO"; else echo "✅ YES"; fi

echo ""
echo "3. Ingest poll() called at runtime?"
INGEST_CALLS=$(grep -r "\.poll()" src --include="*.rs" | grep -v test | wc -l)
if [ "$INGEST_CALLS" -eq 0 ]; then echo "❌ NO"; else echo "✅ YES"; fi

echo ""
echo "4. Profile store updated?"
PROFILE_CALLS=$(grep -r "\.upsert_profile\|\.record_sample\|\.update_health" src --include="*.rs" | grep -v test | wc -l)
if [ "$PROFILE_CALLS" -eq 0 ]; then echo "❌ NO"; else echo "✅ YES"; fi

echo ""
echo "5. UI server exists?"
if [ -f src/bin/ui_server.rs ]; then echo "✅ YES"; else echo "❌ NO"; fi

echo ""
echo "6. proof_run builds?"
if cargo build --release --bin proof_run 2>/dev/null; then echo "✅ YES"; else echo "❌ NO"; fi

echo ""
echo "7. Audit docs in git?"
GIT_DOCS=$(git ls-files docs/ | grep -ic "audit\|smoke" || true)
if [ "$GIT_DOCS" -gt 0 ]; then echo "✅ YES"; else echo "❌ NO"; fi

echo ""
echo "8. Smoke script exists?"
if [ -f scripts/smoke_integrations.sh ]; then echo "✅ YES"; else echo "❌ NO"; fi

echo ""
echo "=== VERDICT ==="
echo "Structure: ✅ (tests pass, code exists)"
echo "Wiring:    ❌ (nothing called at runtime)"
echo "Production-ready: ❌ (missing orchestration)"
```

**To run:**
```bash
chmod +x verify_audit.sh
./verify_audit.sh
```

---

## WHAT WOULD CHANGE THE VERDICT

### To prove EXPORT works:
```bash
grep -r "exporter\|export(" src --include="*.rs" | grep -v test | grep -v "pub fn"
# Would need to return non-empty result
```

### To prove INGEST works:
```bash
grep -r "ingester.poll\|VendorAlertIngester::new" src --include="*.rs" | grep -v test
# Would need to return non-empty result
```

### To prove UI exists:
```bash
ls src/bin/ui_server.rs
# Would need to return the file, not error
```

### To prove docs committed:
```bash
git log --oneline | grep -i "audit\|integration"
# Would need to show commit messages about integration layer
```

### To prove proof_run works:
```bash
cargo build --release --bin proof_run && ./target/release/proof_run --help
# Would need to succeed and show help text
```

---

## AUDIT REPRODUCIBILITY

This audit is **100% reproducible** on any machine with:
- git
- cargo/rust
- bash

Run the commands above in sequence and you'll see the same results.

