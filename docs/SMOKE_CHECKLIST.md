# Manual Smoke Test Checklist

> Pre-release verification checklist for EDR Workbench credibility-hardening phase.
> Run these tests manually before any release.

## Prerequisites

- [ ] `./scripts/tier2_gates.sh --full` passes (all automated tests green)
- [ ] Fresh telemetry data available (or mock data in `telemetry_output/`)
- [ ] Server running: `cargo run --package edr-server`

---

## 1. Selfcheck Endpoint

**Test:** Verify selfcheck returns correct status and actionable errors.

```bash
curl -s http://localhost:3000/api/selfcheck | jq '.verdict'
```

**Expected:**
- Returns `"healthy"`, `"degraded"`, or `"blocked"`
- If `"blocked"`: check `top_issues` array has actionable items
- If `"degraded"`: check `recommended_actions` array

**Failure scenarios to test:**
- [ ] Stop the database → should return `"blocked"` with `db_ok: false`
- [ ] Remove write permissions from `telemetry_output/segments/` → should return `"blocked"`

---

## 2. Export Support Bundle

**Test:** Generate a support bundle and verify it contains expected files.

```bash
curl -X POST http://localhost:3000/api/support-bundle \
  -H "Content-Type: application/json" \
  -d '{"reason": "smoke test", "include_last_hour_events": false}' \
  --output /tmp/support_bundle.zip

unzip -l /tmp/support_bundle.zip
```

**Expected files in bundle:**
- [ ] `manifest.json` - bundle metadata
- [ ] `selfcheck.json` - system diagnostics
- [ ] `config.json` - redacted configuration
- [ ] `events/` directory (if `include_last_hour_events: true`)

---

## 3. Verify Redaction in Bundle

**Test:** Ensure no real usernames or home paths leak into the bundle.

```bash
unzip -p /tmp/support_bundle.zip | grep -iE "/Users/[a-z]|/home/[a-z]|C:\\\\Users\\\\[a-z]" && echo "FAIL: Found unredacted paths" || echo "PASS: No user paths found"
```

**Expected:** "PASS: No user paths found"

---

## 4. Import Bundle (Namespace Isolation)

**Test:** Import a bundle and verify it creates incidents in the `imported_` namespace.

```bash
# If you have a test bundle:
curl -X POST http://localhost:3000/api/bundles/import \
  -F "bundle=@/path/to/test_bundle.zip"

# Check incidents - imported should NOT appear by default
curl -s http://localhost:3000/api/incidents | jq '.incidents[].incident_id'
```

**Expected:**
- [ ] No incident IDs starting with `imported_` in default response
- [ ] Adding `?include_imported=1` shows imported incidents

---

## 5. Namespace Isolation - Individual Endpoints

**Test:** Verify imported incidents are blocked on detail endpoints.

```bash
# Try to access an imported incident without flag (should fail)
curl -s http://localhost:3000/api/incidents/imported_test_inc_001/explain | jq '.narrative'

# Should contain error message about namespace
```

**Expected:** Response includes "imported namespace" error or redirect message.

```bash
# With flag - should work
curl -s "http://localhost:3000/api/incidents/imported_test_inc_001/explain?include_imported=1" | jq '.narrative'
```

---

## 6. Path Traversal Protection

**Test:** Verify evidence deref rejects path traversal attempts.

```bash
# This should be rejected (if you can craft a request with traversal)
# The server should log: [PATH_SAFETY] Blocked unsafe segment path
```

**Check server logs for:**
- [ ] No `../` or `..\\` patterns in successful file reads
- [ ] Any traversal attempts logged with `[PATH_SAFETY]` prefix

---

## 7. PDF/Report Export

**Test:** Generate a PDF report and verify imported namespace is labeled.

```bash
curl -X POST http://localhost:3000/api/reports/generate \
  -H "Content-Type: application/json" \
  -d '{"incident_id": "imported_test_inc_001", "format": "pdf", "include_imported": true}' \
  --output /tmp/report.pdf
```

**Expected:**
- [ ] PDF header includes "IMPORTED BUNDLE" label
- [ ] Incident ID shown with namespace prefix

---

## 8. UI Banner Check

**Test:** Open the web UI and navigate to an imported incident.

1. Navigate to `http://localhost:3000/`
2. Enable "Show Imported" toggle
3. Click on an imported incident

**Expected:**
- [ ] Yellow/orange banner at top: "This incident is from an imported bundle"
- [ ] Incident header shows namespace label

---

## 9. Incident List - Live vs Imported

**Test:** Verify the incident list correctly separates namespaces.

1. Create a live incident (run a probe or wait for real telemetry)
2. Import a bundle with incidents
3. Check `/api/incidents` default response

**Expected:**
- [ ] Only live incidents appear by default
- [ ] Count matches expected live incidents
- [ ] `?include_imported=1` shows all incidents

---

## 10. Full Pipeline Smoke

**Test:** End-to-end test from event to incident to export.

1. Run harmless probe: `curl -X POST http://localhost:3000/api/probe/run`
2. Wait 30 seconds for processing
3. Check for new incident: `curl http://localhost:3000/api/incidents`
4. Export that incident as PDF
5. Generate support bundle

**Expected:**
- [ ] Probe creates detectable telemetry
- [ ] Incident appears with evidence pointers
- [ ] PDF exports successfully
- [ ] Support bundle is valid ZIP

---

## Sign-off

| Item | Status | Tester | Date |
|------|--------|--------|------|
| Selfcheck | ☐ | | |
| Support Bundle | ☐ | | |
| Redaction | ☐ | | |
| Bundle Import | ☐ | | |
| Namespace Isolation | ☐ | | |
| Path Safety | ☐ | | |
| PDF Export | ☐ | | |
| UI Banner | ☐ | | |
| List Separation | ☐ | | |
| Full Pipeline | ☐ | | |

**Release Approved:** ☐ Yes / ☐ No

**Notes:**
```


```
