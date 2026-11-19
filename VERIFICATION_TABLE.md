# COMPONENT VERIFICATION TABLE (STRICT)

| Component | Exists | Builds | Tests | Runtime Wired | Status | Fix Required |
|-----------|--------|--------|-------|---------------|---------|--------------|
| **IncidentExporter** | ✅ `src/integrations/export.rs` | ✅ | ✅ (3 tests pass) | ❌ Never called | UNTESTED IN RUNTIME | Wire in export loop |
| **VendorAlertIngester** | ✅ `src/integrations/ingest.rs` | ✅ | ✅ (5 tests pass) | ❌ poll() never called | UNTESTED IN RUNTIME | Wire in ingest loop |
| **IntegrationProfileStore** | ✅ `src/integrations/profile.rs` | ✅ | ✅ (3 tests pass) | ❌ No runtime updates | UNTESTED IN RUNTIME | Wire in ingest/export loops |
| **ExportSinkConfig** | ✅ `src/integrations/config.rs` | ✅ | ✅ Loads YAML | ❌ Config never read | PARTIAL | Load in export loop |
| **IngestSourceConfig** | ✅ `src/integrations/config.rs` | ✅ | ✅ Loads YAML | ❌ Config never read | PARTIAL | Load in ingest loop |
| **ui_server binary** | ❌ Doesn't exist | N/A | N/A | N/A | MISSING | Create with /api/* routes |
| **proof_run binary** | ✅ `src/bin/proof_run.rs` | ❌ (edr-core path broken) | N/A | N/A | BROKEN | Fix edr-core path |
| **Determinism test** | ✅ `test_deterministic_export()` | ✅ | ✅ Passes | ? (only in-memory) | PARTIAL | Verify disk output determinism |
| **Smoke test script** | ❌ Doesn't exist | N/A | N/A | N/A | MISSING | Create `scripts/smoke_integrations.sh` |
| **Audit documents** | ✅ 8 docs in `docs/` | N/A | N/A | N/A | NOT COMMITTED | `git add docs/*AUDIT*.md && git commit` |
| **Event/Fact Pipeline** | ✅ Defined in hypothesis/ | ✅ | ✅ Multiple tests | ❌ No integration loop | PARTIAL | Create event loop |

---

## REPRODUCIBLE COMMANDS

### Build test (to verify current broken state):
```bash
cd /Users/jermainebaker/git-glue
cargo build --release --bin proof_run 2>&1 | tail -5
# Expected output shows edr-core path error
```

### Run tests (to verify existing tests pass):
```bash
cd /Users/jermainebaker/git-glue
cargo test --lib 2>&1 | grep "test result"
# Output: test result: ok. 201 passed; 0 failed
```

### Check IncidentExporter wiring:
```bash
cd /Users/jermainebaker/git-glue
grep -r "IncidentExporter::new" src --include="*.rs" | grep -v test | wc -l
# Output: 0 (not called at runtime)
```

### Check Ingest loop:
```bash
cd /Users/jermainebaker/git-glue
grep -r "\.poll()" src --include="*.rs" | grep -v test | wc -l
# Output: 0 (poll never called at runtime)
```

### Check Profile store updates:
```bash
cd /Users/jermainebaker/git-glue
grep -r "\.upsert_profile\|\.record_sample\|\.update_health" src --include="*.rs" | grep -v test | wc -l
# Output: 0 (never updated at runtime)
```

### Check ui_server:
```bash
cd /Users/jermainebaker/git-glue
ls src/bin/ui_server.rs 2>/dev/null || echo "MISSING"
# Output: MISSING
```

### Check audit docs committed:
```bash
cd /Users/jermainebaker/git-glue
git ls-files | grep "docs/.*AUDIT" | wc -l
# Output: 0 (not in git)
```

---

## ROOT CAUSE

The previous task completion claims were based on:
- ✅ Code structure being created
- ✅ Tests being written  
- ✅ Unit tests passing

But **NOT verified**:
- ❌ Runtime instantiation
- ❌ Integration loops
- ❌ API endpoints  
- ❌ Smoke tests
- ❌ Git commits

This is a **proof-of-structure without proof-of-wiring**.

---

## MINIMUM VIABLE FIX (1 day)

To make claims TRUE, implement:

1. **Fix edr-core path** (30 min)
   - Create `edr-core/Cargo.toml` with stubs
   - Fix root `Cargo.toml` dependency path
   
2. **Add export loop** (30 min)
   - Create background task in `src/main.rs`
   - Call `exporter.export_batch()` periodically
   
3. **Add ingest loop** (30 min)
   - Create background task in `src/main.rs`
   - Call `ingester.poll()` continuously
   
4. **Create ui_server** (1 hour)
   - Create `src/bin/ui_server.rs`
   - Expose `/api/integrations` route
   - Expose `/api/capabilities` route
   
5. **Smoke test script** (30 min)
   - Create `scripts/smoke_integrations.sh`
   - Test all 3 endpoints
   
6. **Commit docs** (5 min)
   - `git add docs/*AUDIT*.md`
   - `git commit -m "docs: integration layer audit"`

**Total: 3-4 hours of actual implementation**

