# macOS Model A - Production EDR Pipeline Specification

## Non-Negotiable Invariants

### 0.1 TELEMETRY_ROOT Single-Source-of-Truth
All binaries resolve identically with strict precedence:
1. CLI: `--telemetry-root <path>`
2. Env: `EDR_TELEMETRY_ROOT`
3. Default: `./telemetry_output` (cwd-relative)

**NO** current_exe() walking, repo-root assumptions, hardcoded /tmp paths, or fallbacks.

### 0.2 Evidence as First-Class Dereferenceable Pointers
Every incident must preserve ability to reconstruct:
- Which raw events caused each slot fill
- Timeline of dereferenced raw events
- Narrative citing EvidencePtrs

**EvidencePtr minimum:**
- `segment_id` (stable)
- `record_index` (0-based line number in JSONL)
- `ts_ms` (event timestamp)
- `event_type` (for validation)
- optional: `event_sha256` (integrity)

### 0.3 Idempotency & Restart Safety
- locald must not reinsert events on restart
- locald must not duplicate incidents on replay
- Implement: `locald_state` SQLite table with checkpoint tracking
- Segment rotation must be atomic (temp → fsync → rename)

### 0.4 Deterministic Incident Identity
```
incident_id = SHA256(playbook_id || primary_entity || ts_bucket)
where ts_bucket = (ts_ms / 3600000) * 3600000  // 1-hour buckets
```
Upsert updates: ts_updated, slots_json, evidence_json, tags_json, confidence.

---

## 1. Capture: Real OpenBSM Ingestion

### 1.1 Requirements
- Run as root; validate at startup
- Read `/dev/auditpipe` continuously
- Parse events robustly (unknown record types → skip + log)
- Emit normalized JSON to JSONL segments
- Write segments atomically (temp → fsync → rename)
- Maintain bounded resource usage

### 1.2 Normalized Event Schema (versioned)

```json
{
  "schema_version": 1,
  "ts_ms": 1766214709000,
  "host": "hostname",
  "event_type": "proc_exec|file_write|file_create|file_delete|net_connect|auth|privilege",
  "subject": {
    "uid": 501,
    "gid": 20,
    "auid": 501,
    "euid": 501,
    "egid": 20,
    "session_id": 12345
  },
  "process": {
    "pid": 1234,
    "ppid": 1,
    "exe": "/usr/bin/curl",
    "argv": ["curl", "http://example.com"],
    "cwd": "/tmp",
    "signing": null
  },
  "raw_bsm_debug": { }
}
```

### 1.3 Event-Type Specific Fields
- **proc_exec**: exe, argv, ppid, parent_exe, env (optional)
- **file_write**: path, inode (optional), bytes
- **file_create**: path
- **file_delete**: path
- **net_connect**: dst_ip, dst_port, proto
- **auth**: result, method, account
- **privilege**: transition, from_uid, to_uid, mechanism

### 1.4 Segment & Index Format

**Location:** `${TELEMETRY_ROOT}/segments/<segment_id>.jsonl`

**Index:** `${TELEMETRY_ROOT}/index.json`
```json
{
  "segments": [
    {
      "segment_id": "seg_1766214709",
      "rel_path": "segments/seg_1766214709.jsonl",
      "ts_first": 1766214709000,
      "ts_last": 1766214800000,
      "records": 4521,
      "sha256": "abc123...",
      "size_bytes": 2097152
    }
  ],
  "last_updated_ts": 1766214800000
}
```

---

## 2. Ingest: edr_locald Deterministic Incident Compiler

### 2.1 Database Schema

**telemetry_events:**
```sql
CREATE TABLE telemetry_events (
  id INTEGER PRIMARY KEY,
  ts_ms INTEGER NOT NULL,
  segment_id TEXT NOT NULL,
  segment_rel_path TEXT,
  record_index INTEGER NOT NULL,
  event_type TEXT NOT NULL,
  host TEXT,
  pid INTEGER,
  uid INTEGER,
  event_json TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(segment_id, record_index)
);
CREATE INDEX idx_telemetry_ts ON telemetry_events(ts_ms);
CREATE INDEX idx_telemetry_event_type_ts ON telemetry_events(event_type, ts_ms);
```

**incidents:**
```sql
CREATE TABLE incidents (
  incident_id TEXT PRIMARY KEY,
  playbook_id TEXT NOT NULL,
  primary_entity TEXT,
  ts_created INTEGER NOT NULL,
  ts_updated INTEGER NOT NULL,
  severity INTEGER,
  confidence REAL,
  title TEXT,
  tags_json TEXT,
  slots_json TEXT,
  evidence_json TEXT,
  narrative_json TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_incidents_ts_created ON incidents(ts_created);
CREATE INDEX idx_incidents_ts_updated ON incidents(ts_updated);
CREATE INDEX idx_incidents_playbook ON incidents(playbook_id);
CREATE INDEX idx_incidents_severity ON incidents(severity);
```

**locald_state (checkpoint for restart safety):**
```sql
CREATE TABLE locald_state (
  key TEXT PRIMARY KEY,
  value TEXT,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
-- Checkpoint entries: segment_id -> last_record_index, ts
```

### 2.2 Ingest Loop

```
Load playbooks from ./playbooks/macos/ or ${TELEMETRY_ROOT}/playbooks/macos/
Open analysis.db
Watch index.json for new segments
  For each new segment (dedup by segment_id, check locald_state):
    Open segment JSONL
    For each line (record_index):
      Parse JSON -> event
      INSERT INTO telemetry_events (idempotent: UNIQUE constraint)
      extract_facts_from_event() -> Vec<Fact>
      feed_slot_engine(facts)
      IF playbook fires:
        compute deterministic incident_id
        UPSERT incident (INSERT OR REPLACE)
        store evidence_json (slot -> EvidencePtr list)
        store narrative_json
    UPDATE locald_state (checkpoint: segment_id, last_record_index)
  Sleep 500ms, loop
```

### 2.3 Incident UPSERT Model

**incident_id determination:**
```
incident_id = SHA256(
  playbook_id || 
  primary_entity_key ||
  (ts_ms / 3600000) * 3600000
)
primary_entity_key = host || ":" || user (derived from first fact)
```

**Upsert logic:**
```sql
INSERT INTO incidents (incident_id, playbook_id, primary_entity, 
  ts_created, ts_updated, severity, title, tags_json, 
  slots_json, evidence_json, narrative_json)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(incident_id) DO UPDATE SET
  ts_updated = ?,
  slots_json = json_merge_patch(slots_json, ?),
  evidence_json = json_merge_patch(evidence_json, ?),
  tags_json = ?,
  severity = ?;
```

---

## 3. UI Server: Evidence-Backed API

### 3.1 /api/health
Return true only if:
- TELEMETRY_ROOT resolved
- segments dir exists
- index.json readable
- analysis.db readable
- last_ingest_time from locald_state

### 3.2 /api/incidents
Query real rows, include: incident_id, playbook_id, severity, ts_created, primary_entity.

### 3.3 /api/incidents/:id/explain (CRITICAL)

Must:
1. Load incident row
2. Parse evidence_json -> Vec<EvidencePtr>
3. For each EvidencePtr:
   - Load raw event from segment JSONL at record_index
   - Return {ptr, raw_event}
4. Build timeline: sorted by ts_ms, cite each event
5. Return: {incident_id, narrative, evidence, timeline}

If deref fails: include explicit error per ptr.

---

## 4. Detection Content: Production macOS Playbooks

**Minimum viable set:**

### 4.1 User-Writable Exec + Egress
```
Slots:
  1. proc_exec from /tmp, /var/tmp, ~/Downloads, etc
  2. net_connect within 30s
Entity: per-user, per-host
TTL: 30 seconds
Tags: [TA0001 Initial Access, TA0008 Lateral Movement]
Severity: HIGH
```

### 4.2 LaunchAgent/Daemon Persistence
```
Slots:
  1. file_write to ~/Library/LaunchAgents/ or /Library/LaunchDaemons/
  2. proc_exec of launchctl (load/bootstrap) within 60s
Entity: per-host (daemons) or per-user (agents)
TTL: 60 seconds
Tags: [TA0003 Persistence]
Severity: CRITICAL
```

### 4.3 LoTL Chain (osascript or shell)
```
Slots:
  1. proc_exec of /usr/bin/osascript or /bin/sh with suspicious args (eval, curl, etc)
  2. net_connect OR file_write within 10s
Entity: per-user
TTL: 10 seconds
Tags: [TA0005 Defense Evasion, TA0007 Discovery]
Severity: HIGH
```

### 4.4 Credential Prompt / Auth Anomaly
```
Slots:
  1. Multiple auth failures (>3 within 60s)
  2. OR sudo/su to privilege with immediate proc_exec
Entity: per-user, per-host
TTL: 60 seconds
Tags: [TA0004 Privilege Escalation]
Severity: MEDIUM
```

---

## 5. Testing & Validation

### 5.1 Test Matrix
**Location:** `scripts/macos/run_detection_matrix.sh`

Executes tests from Atomic MITRE ATT&CK where feasible, else scripted equivalents.

**Test loop:**
1. Set TELEMETRY_ROOT=/tmp/modela_test_<timestamp>
2. Start capture (sudo), locald, ui_server
3. Execute test (Atomic or script)
4. Wait for ingest
5. Query: /api/incidents, /api/incidents/:id/explain
6. Collect results

### 5.2 Gap Report Artifact
**Output:** `${TELEMETRY_ROOT}/reports/<ts>/gap_report.md` and `.csv`

**Row format:**
| Technique | Test ID | Expected Events | Observed Events | Incident Fired? | Root Cause | Remediation |
|-----------|---------|-----------------|-----------------|-----------------|-----------|-------------|
| User-writable exec | ATT&CK_T1059 | proc_exec,net_connect | 2/2 | YES | - | OK |
| LaunchAgent write | ATT&CK_T1547 | file_write | 0/1 | NO | CAPTURE GAP | Need IOKit kqueue for LaunchAgents path |

**Root cause categories:**
- CAPTURE GAP: OpenBSM doesn't expose
- PARSE GAP: Capture saw it, normalization failed
- FACT GAP: Events ok, facts missing
- PLAYBOOK GAP: Facts ok, no matching rule
- EXPLAIN GAP: Incident exists, deref broken

---

## 6. Operationalization

### 6.1 launchd Packaging

**Files to create:**
- `scripts/macos/install_launchd.sh` - install plists and binaries
- `scripts/macos/uninstall_launchd.sh` - remove plists and state
- `config/com.edr.capture.plist` - root daemon for capture
- `config/com.edr.locald.plist` - user agent for locald
- `config/com.edr.ui_server.plist` - user agent for ui_server

Each plist must set TELEMETRY_ROOT consistently (env var or args).

### 6.2 Logging & Robustness
- Structured logs: json/text with level
- No panics on malformed records (skip + log)
- Bounded buffers, backpressure if locald lags
- Segment retention policy documented

### 6.3 Perf Baseline Script
**Location:** `scripts/macos/smoke_test.sh`

Measure:
- CPU/RSS snapshots
- Event throughput (events/sec)
- Segment growth rate
- End-to-end latency (event arrival → incident firing)

---

## 7. Verification Deliverables

### 7.1 File-by-File Map (call chains)

### 7.2 Hard Proof Commands (must pass)

### 7.3 Truthful Status Bullets
- ✅ Implemented now
- 🟨 Not implemented / Known limitations / Next fixes

---

## Production Non-Negotiables Summary

1. **Real OpenBSM** - No demos, no feed modes (except behind off-by-default FEATURE_DEMO)
2. **Evidence-first** - Every incident dereferenceable to raw JSON
3. **Deterministic IDs** - Idempotent upsert, restart-safe
4. **Production schema** - Checkpoint table, proper indexes
5. **Serious playbooks** - Real macOS techniques, MITRE tags, gap analysis
6. **Engineer-grade testing** - Coverage matrix, gap report, remediation items
7. **Operationalization** - launchd, logging, perf baseline
8. **Proof commands** - Build, run, curl, sqlite3 - all provided with expected outputs

