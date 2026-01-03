# INTEGRATION WIRING FIX PLAN

## Status: REPO IN BROKEN INTERMEDIATE STATE

The restructuring from `crates/locald + crates/core` → `src/` is incomplete:
- ✅ Code moved to src/integrations/, src/os/, etc.
- ❌ Cargo.toml still references `path = "../core"` 
- ❌ edr-core crate no longer exists
- ❌ 10+ modules import from edr_core (Event, EvidencePtr, etc.)
- ❌ Build fails at dependency resolution

## MINIMAL FIX (2 options)

### OPTION A: Remove edr-core dependency (UNSAFE - breaks imports)

**Problem:** 10+ files import `use edr_core::Event` but code is nowhere else in repo

### OPTION B: Create stub edr-core crate (CORRECT)

**Solution:** Create a minimal `edr-core/` crate with re-exports from this crate.

**Files to create:**
1. `edr-core/Cargo.toml`
2. `edr-core/src/lib.rs` (re-exports)

**edr-core/Cargo.toml:**
```toml
[package]
name = "edr-core"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }

[lib]
name = "edr_core"
path = "src/lib.rs"
```

**edr-core/src/lib.rs** (minimal):
```rust
//! EDR Core - Canonical types for event processing
//!
//! Re-exports types from edr-locald that were historically in edr-core

pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::collections::BTreeMap;

/// Placeholder Event type (historically in edr-core, now needs proper definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts_ms: i64,
    pub host: String,
    pub tags: Vec<String>,
    pub fields: BTreeMap<String, serde_json::Value>,
}

/// Placeholder EvidencePtr type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePtr {
    pub segment_id: String,
    pub offset: u64,
    pub hash: String,
}

/// Placeholder Incident type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub incident_id: String,
    pub host_id: String,
    pub family: String,
    pub severity: String,
    pub confidence: f64,
}
```

**Cargo.toml change (root):**
```diff
[dependencies]
-edr-core = { path = "../core" }
+edr-core = { path = "edr-core" }
```

---

## RUNTIME WIRING REQUIREMENTS

After fixing the build, these components need to be connected:

### 1. EXPORT LOOP (Main or API Handler)
**File:** `src/main.rs` or API route handler  
**Code pattern:**
```rust
async fn export_incidents() {
    let config = ExportSinkConfig::load_yaml("config/export.yaml").ok()?;
    let mut exporter = IncidentExporter::new(config)?;
    
    loop {
        // Get incidents from hypothesis controller
        let incidents = hypothesis_controller.get_all_incidents();
        
        // Export in batch
        if !incidents.is_empty() {
            exporter.export_batch(&incidents, None)?;
        }
        
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

### 2. INGEST LOOP (Background Task)
**File:** `src/main.rs` or background task  
**Code pattern:**
```rust
async fn run_ingest() {
    let config = IngestSourceConfig::load_yaml("config/ingest.yaml").ok()?;
    let mut ingester = VendorAlertIngester::new(config)?;
    let mut profiles = IntegrationProfileStore::default();
    
    loop {
        // Poll all sources
        match ingester.poll() {
            Ok(result) => {
                for fact in result.vendor_alert_facts {
                    // Record sample in profile store
                    profiles.record_sample_event(&result.source_id, &fact);
                    
                    // Emit into hypothesis pipeline
                    hypothesis_controller.add_fact(fact)?;
                }
                
                // Update health status
                profiles.update_health_status(&result.source_id, result.health);
            }
            Err(e) => {
                profiles.record_error(&result.source_id, &e);
            }
        }
        
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
```

### 3. UI ENDPOINTS (api/integrations)
**File:** Need to create `src/bin/ui_server.rs`  
**Routes needed:**
```rust
// GET /api/integrations
pub async fn get_integrations(
    State(profiles): State<Arc<IntegrationProfileStore>>
) -> Json<Vec<IntegrationProfileResponse>> {
    profiles.list_profiles().into()
}

// GET /api/capabilities  
pub async fn get_capabilities(
    State(profiles): State<Arc<IntegrationProfileStore>>
) -> Json<CapabilityMatrix> {
    profiles.merged_capabilities().into()
}

// GET /api/integrations/:id/sample
pub async fn get_sample_events(
    Path(id): Path<String>,
    State(profiles): State<Arc<IntegrationProfileStore>>
) -> Json<Vec<SampleEvent>> {
    profiles.get_recent_samples(&id).into()
}
```

---

## SMOKE TEST SCRIPT

**File:** `scripts/smoke_integrations.sh`
```bash
#!/bin/bash
set -e

# Start ui_server
./target/release/ui_server --port 8080 &
SERVER_PID=$!

# Give server time to start
sleep 2

# Test /api/integrations
echo "Testing GET /api/integrations..."
curl -s http://localhost:8080/api/integrations | jq . > /tmp/integrations.json
if [ ! -s /tmp/integrations.json ]; then
    echo "❌ FAIL: /api/integrations returned empty"
    kill $SERVER_PID
    exit 1
fi
echo "✅ PASS: /api/integrations returned data"

# Test /api/capabilities
echo "Testing GET /api/capabilities..."
curl -s http://localhost:8080/api/capabilities | jq . > /tmp/capabilities.json
if [ ! -s /tmp/capabilities.json ]; then
    echo "❌ FAIL: /api/capabilities returned empty"
    kill $SERVER_PID
    exit 1
fi
echo "✅ PASS: /api/capabilities returned data"

# Test /api/integrations/:id/sample
FIRST_ID=$(jq -r '.[0].integration_id' /tmp/integrations.json 2>/dev/null || echo "")
if [ -n "$FIRST_ID" ]; then
    echo "Testing GET /api/integrations/$FIRST_ID/sample..."
    curl -s http://localhost:8080/api/integrations/$FIRST_ID/sample | jq . > /tmp/sample.json
    if [ ! -s /tmp/sample.json ]; then
        echo "❌ FAIL: /api/integrations/:id/sample returned empty"
        kill $SERVER_PID
        exit 1
    fi
    echo "✅ PASS: /api/integrations/:id/sample returned data"
fi

# Clean up
kill $SERVER_PID
echo ""
echo "✅ ALL SMOKE TESTS PASSED"
```

---

## proof_run FIX

Once edr-core is restored, proof_run will build:

```bash
$ cargo build --release --bin proof_run
Finished `release` profile [optimized] target/release/proof_run

$ ./target/release/proof_run --telemetry-root /var/lib/edr --out /tmp/proof.json

✅ Proof run complete: /tmp/proof.json

Wiring Check Results:
  IntegrationProfileStore: ✓
  ExportSinkConfig: ✓
  IngestSourceConfig: ✓

Overall Status: ALL SYSTEMS WIRED
```

---

## ACTUAL FIX CHECKLIST

- [ ] 1. Create `edr-core/Cargo.toml` with minimal stubs
- [ ] 2. Create `edr-core/src/lib.rs` with re-exports  
- [ ] 3. Update root `Cargo.toml`: `edr-core = { path = "edr-core" }`
- [ ] 4. Verify `cargo build` succeeds
- [ ] 5. Implement export loop in src/main.rs
- [ ] 6. Implement ingest loop in src/main.rs
- [ ] 7. Create src/bin/ui_server.rs with /api/* routes
- [ ] 8. Create scripts/smoke_integrations.sh
- [ ] 9. Run smoke tests and verify all endpoints respond
- [ ] 10. Commit: `git add -A && git commit -m "Fix: Complete integration layer wiring"`

---

## ESTIMATED EFFORT

- edr-core stub: 30 min
- Export/ingest loops: 1 hour  
- UI server: 1.5 hours
- Smoke tests: 30 min
- **Total: 3.5 hours**

