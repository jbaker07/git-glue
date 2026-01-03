# Integration Layer Documentation

This document describes the Integration Layer for the EDR incident compiler,
enabling bidirectional data flow with external SIEM/open-stack tools.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Integration Layer                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────┐                      ┌──────────────────────────────┐ │
│  │   EXPORT PATH    │                      │      INGEST PATH             │ │
│  │                  │                      │                              │ │
│  │  IncidentStore   │                      │   Wazuh alerts.json          │ │
│  │       │          │                      │   Zeek/Suricata EVE          │ │
│  │       ▼          │                      │   Generic JSONL              │ │
│  │  IncidentExporter│                      │         │                    │ │
│  │       │          │                      │         ▼                    │ │
│  │       ▼          │                      │   VendorAlertIngester        │ │
│  │  JSONL Files     │                      │         │                    │ │
│  │  (Wazuh/Elastic) │                      │         ▼                    │ │
│  │                  │                      │   VendorAlertFact            │ │
│  │  Schema v1.0:    │                      │   (FactProvenance::Soft)     │ │
│  │  - incident_id   │                      │         │                    │ │
│  │  - host_id       │                      │         ▼                    │ │
│  │  - namespace     │                      │   Soft Join by IP/Time       │ │
│  │  - family        │                      │   → Enrich Existing Incident │ │
│  │  - severity      │                      │                              │ │
│  │  - timeline_summary                     │                              │ │
│  │  - evidence_summary                     │                              │ │
│  │  - join_keys     │                      │                              │ │
│  └──────────────────┘                      └──────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Enable Export

Edit `config/integrations.yaml`:

```yaml
export:
  enabled: true
  sinks:
    - sink_id: wazuh
      sink_type: jsonl_file
      output_path: /var/lib/edr/exports/wazuh
      min_severity: medium
      include_tier0: true
```

Incidents will be written to:
```
/var/lib/edr/exports/wazuh/incidents.jsonl
```

### 2. Enable Ingest

```yaml
ingest:
  enabled: true
  sources:
    - source_id: wazuh_alerts
      source_type: wazuh_alerts
      input_path: /var/ossec/logs/alerts/alerts.json
      vendor: wazuh
      join_strategy: ip_time_window
      join_window_seconds: 300
```

### 3. Run Metrics Harness

```bash
# Basic report
cargo run --bin integration_metrics -- \
  --incidents-dir /var/lib/edr/incidents

# With per-incident breakdown
cargo run --bin integration_metrics -- \
  --incidents-dir /var/lib/edr/incidents \
  --per-incident

# Include ingest stats
cargo run --bin integration_metrics -- \
  --incidents-dir /var/lib/edr/incidents \
  --ingest /var/ossec/logs/alerts/alerts.json
```

## Reference Integrations

### Wazuh

**Export**: Incidents → JSONL → Wazuh `local_decoder.xml` / Filebeat

**Ingest**: Wazuh `alerts.json` → VendorAlertFact → Soft join by IP+time

Wazuh alert fields mapped:
| Wazuh Field | VendorAlertFact Field |
|-------------|----------------------|
| `agent.ip` | `ip_indicators[].ip` |
| `rule.id` | `original_event_id` |
| `rule.mitre.id` | `mitre_tags` |
| `timestamp` | `timestamp` |
| `data.win.system.processId` | `process_hints[].pid` |
| `data.win.system.image` | `process_hints[].exe` |

### Zeek / Suricata EVE

**Ingest Only**: EVE JSON → VendorAlertFact → Soft join for network enrichment

Zeek EVE fields mapped:
| Zeek Field | VendorAlertFact Field |
|------------|----------------------|
| `id.orig_h` | `ip_indicators[].ip` (src) |
| `id.resp_h` | `ip_indicators[].ip` (dst) |
| `query` (DNS) | `dns_query` |
| `uid` | `original_event_id` |
| `ts` | `timestamp` |

## Export Schema (v1.0)

```json
{
  "schema_version": "1.0",
  "incident_id": "a1b2c3d4...",
  "host_id": "hostname-or-uuid",
  "namespace": "endpoint.windows",
  "family": "credential_access",
  "severity": "critical",
  "created_at": "2025-01-15T12:34:56Z",
  "updated_at": "2025-01-15T12:35:00Z",
  "timeline_summary": [
    {
      "fact_type": "process_exec",
      "timestamp": "2025-01-15T12:34:56Z",
      "scope_key_prefix": "abc123...",
      "summary_fields": {
        "exe": "mimikatz.exe",
        "pid": "1234"
      }
    }
  ],
  "evidence_summary": {
    "playbook_names": ["T1003_LSASS"],
    "hypothesis_scores": { "credential_dumping": 0.95 },
    "tier0_violations": ["lsass_access"]
  },
  "join_keys": {
    "host": "hostname",
    "ips": ["192.168.1.100"],
    "exe_hashes": ["sha256:abc..."],
    "time_start": "2025-01-15T12:34:00Z",
    "time_end": "2025-01-15T12:35:00Z"
  }
}
```

## VendorAlertFact Structure

```rust
pub struct VendorAlertFact {
    pub vendor: String,           // "wazuh", "zeek", "suricata"
    pub source: String,           // source_id from config
    pub original_event_id: String,
    pub timestamp: DateTime<Utc>,
    pub provenance: FactProvenance, // Always Soft for ingested
    
    // Join hints
    pub ip_indicators: Vec<IpIndicator>,
    pub process_hints: Vec<ProcessHint>,
    pub file_hints: Vec<FileHint>,
    pub dns_query: Option<String>,
    
    // Enrichment
    pub mitre_tags: Vec<String>,
    pub raw_payload: serde_json::Value,
}
```

## Join Strategies

| Strategy | Description | Use Case |
|----------|-------------|----------|
| `ip_time_window` | Match IP + time within window | Network alerts → host incidents |
| `host_time_window` | Match hostname + time | Host-based alerts |
| `exe_hash` | Match by SHA256 of executable | File-based correlation |
| `none` | No automatic join | Manual correlation only |

## Severity Levels

| Level | Numeric | Export Default |
|-------|---------|----------------|
| `info` | 0 | No |
| `low` | 1 | No |
| `medium` | 2 | Yes |
| `high` | 3 | Yes |
| `critical` | 4 | Yes |

**Note**: Tier-0 invariants (`memory_rwx`, `lsass`, `credential_access`, `rootkit`)
are **always exported** regardless of severity threshold.

## Metrics Report

The metrics harness generates:

1. **JSON Report**: `/var/lib/edr/metrics/run_<id>.json`
2. **Summary**: `/var/lib/edr/metrics/run_<id>_summary.txt`

### Evidence Quality Score (EQS)

```
EQS = 0.25 × timeline_score 
    + 0.35 × evidence_score 
    + 0.20 × entity_score 
    + 0.20 × lineage_score

Where:
  timeline_score = min(1.0, timeline_facts / 10)
  evidence_score = min(1.0, (playbooks × 0.3 + tier0 × 0.5 + hypotheses × 0.2))
  entity_score   = min(1.0, unique_entities / 5)
  lineage_score  = min(1.0, parent_refs / 3)
```

### Sample Output

```
═══════════════════════════════════════════════════════════════════════
                      INTEGRATION METRICS REPORT
═══════════════════════════════════════════════════════════════════════
Report ID:        run_1705312345_abcdef
Generated:        2025-01-15T12:34:56Z

INCIDENT SUMMARY
────────────────
Total Incidents:        42
By Severity:
  critical:              3
  high:                 12
  medium:               20
  low:                   7
Average EQS:          0.73

EXPORT METRICS
────────────────
Exported:               35
Filtered (severity):     7
Compression Ratio:     12.3x  (raw events → incidents)
Duplicate IDs:           0

INGEST METRICS
────────────────
Alerts Processed:      156
Alerts Joined:          89  (57.1%)
Join Strategy:    ip_time_window
```

## File Locations

| Purpose | Path |
|---------|------|
| Config | `config/integrations.yaml` |
| Export Output | `/var/lib/edr/exports/<sink_id>/incidents.jsonl` |
| Metrics Output | `/var/lib/edr/metrics/run_<id>.json` |
| Wazuh Input | `/var/ossec/logs/alerts/alerts.json` |
| Zeek Input | `/var/log/zeek/current/*.log` |

## Acceptance Criteria

The integration layer satisfies these acceptance tests:

1. ✅ **Export creates JSONL with correct schema**
   - Schema version present
   - All required fields populated
   - JSONL format (one JSON per line)

2. ✅ **Tier-0 always exported**
   - `memory_rwx`, `lsass`, `credential_access`, `rootkit` families
   - Exported regardless of severity threshold

3. ✅ **Wazuh ingest produces stable scope key**
   - Deterministic: same alert → same scope_key
   - Format: `vendoralert:<vendor>:<host>:<time_bucket>`

4. ✅ **Zeek EVE DNS produces fact with query**
   - DNS queries extracted into `dns_query` field
   - Available for enrichment join

## Module Structure

```
crates/locald/src/integrations/
├── mod.rs           # Module exports
├── config.rs        # Configuration structures
├── vendor_alert.rs  # VendorAlertFact type
├── export.rs        # IncidentExporter
├── ingest.rs        # VendorAlertIngester
└── metrics.rs       # MetricsReport, EQS calculation

src/bin/
└── integration_metrics.rs  # CLI harness

tests/
└── integration_acceptance.rs  # Acceptance tests
```

## Running Tests

```bash
# All integration tests
cargo test integration_acceptance

# Specific test
cargo test test_export_creates_jsonl_with_correct_schema

# With output
cargo test integration_acceptance -- --nocapture
```
