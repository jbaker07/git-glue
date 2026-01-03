# Integration Profiles + UI Surfacing

This document describes the Integration Profiles system that provides first-class integration metadata visibility in the EDR dashboard.

## Overview

Integration Profiles enable:
- **First-class integration metadata** (type, mode, facts supported, join keys, health)
- **Capabilities Matrix visualization** (FactTypes × Sources with HARD/SOFT/NONE fidelity)
- **Deterministic + auditable mapping** with provenance tracking
- **Vendor-agnostic design** supporting multiple integration types

## Data Model

### IntegrationProfile

```rust
pub struct IntegrationProfile {
    pub integration_id: String,
    pub name: String,
    pub integration_type: IntegrationType,
    pub mode: IntegrationMode,
    pub enabled: bool,
    pub facts_supported: HashMap<String, Fidelity>,
    pub join_keys_supported: JoinKeySupport,
    pub mapping_version: String,
    pub last_seen_ts: Option<DateTime<Utc>>,
    pub eps: f64,                    // Events per second
    pub parse_error_rate: f64,      // 0.0 - 1.0
    pub events_processed: u64,
    pub facts_created: u64,
    pub health_status: HealthStatus,
}
```

### Fidelity Levels

| Level | Description | Example |
|-------|-------------|---------|
| **HARD** | Full telemetry, direct observation | macOS ES `exec` events |
| **SOFT** | Partial/joined, inferred context | Wazuh alerts joined to process |
| **NONE** | Not supported by this source | Network monitor can't see files |

### Integration Types

- `Wazuh` - Wazuh HIDS
- `Zeek` - Zeek network monitor
- `Suricata` - Suricata IDS
- `Osquery` - osquery fleet
- `Elastic` - Elastic SIEM
- `Splunk` - Splunk
- `Syslog` - Generic syslog
- `JsonlFile` - JSONL file export
- `Custom(String)` - Custom integration

### Integration Modes

- `Export` - Send facts to external system
- `Ingest` - Import events from external system
- `Both` - Bidirectional integration

### Join Key Support

```rust
pub struct JoinKeySupport {
    pub proc_key: bool,        // Process key (pid + start_time)
    pub file_key: bool,        // File key (inode/path)
    pub socket_key: bool,      // Socket key (5-tuple)
    pub identity_key: bool,    // User/identity key
    pub dns_attribution: bool, // DNS → process attribution
    pub thread_key: bool,      // Thread-level granularity
}
```

## API Endpoints

### GET /api/integrations

List all integration profiles with health and stats.

**Query Parameters:**
- `mode` - Filter by mode (`export`, `ingest`, `both`)
- `enabled_only` - Only show enabled integrations

**Response:**
```json
{
  "integrations": [
    {
      "integration_id": "wazuh_main",
      "name": "Wazuh HIDS",
      "integration_type": "wazuh",
      "mode": "both",
      "enabled": true,
      "health_status": "healthy",
      "eps": 42.5,
      "parse_error_rate": 0.002,
      "facts_supported_count": 7,
      "join_keys_supported_count": 4,
      "last_seen_ts": "2024-01-15T12:34:56Z"
    }
  ],
  "total": 1
}
```

### GET /api/integrations/:id

Get detailed integration profile.

**Response:**
```json
{
  "integration": {
    "summary": { ... },
    "facts_supported": {
      "exec": "soft",
      "vendor_alert": "hard"
    },
    "join_keys_supported": {
      "proc_key": true,
      "file_key": true,
      "socket_key": true
    }
  }
}
```

### GET /api/capabilities

Get merged capabilities matrix for all sources.

**Query Parameters:**
- `include_collectors` - Include native collectors (default: true)
- `host_id` - Filter by host
- `namespace` - Filter by namespace

**Response:**
```json
{
  "sources": [
    {
      "id": "macos_endpoint_security",
      "name": "macOS ES",
      "source_type": "collector",
      "health_status": "healthy"
    },
    {
      "id": "wazuh_main",
      "name": "Wazuh HIDS",
      "source_type": "integration",
      "health_status": "healthy"
    }
  ],
  "fact_support": {
    "exec": {
      "macos_endpoint_security": "hard",
      "wazuh_main": "soft"
    },
    "dns_resolve": {
      "macos_endpoint_security": "hard",
      "zeek_network": "hard"
    }
  },
  "join_key_support": {
    "proc_key": {
      "macos_endpoint_security": true,
      "wazuh_main": true,
      "zeek_network": false
    }
  }
}
```

### GET /api/integrations/:id/sample

Get sample raw + mapped events for mapping inspection.

**Query Parameters:**
- `limit` - Max samples to return (default: 10)

**Response:**
```json
{
  "integration_id": "wazuh_main",
  "samples": [
    {
      "raw_event_id": "wazuh_1234567",
      "raw_json_hash": "a1b2c3d4e5f6",
      "mapping_version": "wazuh_4.x",
      "mapped_at": "2024-01-15T12:34:56Z",
      "raw_event_summary": "{...}",
      "mapped_event": {
        "fact_type": "vendor_alert",
        "vendor": "wazuh"
      },
      "derived_scope_keys": [
        {
          "key": "vendoralert:wazuh:win-dc-01:28401",
          "key_type": "campaign",
          "fidelity": "soft",
          "join_confidence": 0.85,
          "confidence_reason": "IP+time window match"
        }
      ]
    }
  ],
  "total_available": 125
}
```

## UI Components

### Integrations Page

The Integrations page (`/ui/index.html#integrations`) displays:

1. **Integration Profiles Table**
   - Status indicator (🟢 healthy / 🟡 warning / 🔴 error)
   - Name, type, mode
   - EPS (events per second)
   - Error rate (% parse failures)
   - Facts supported count
   - Join keys supported count
   - Last seen timestamp
   - Actions (view samples)

2. **Capabilities Matrix**
   - Rows: Fact types (exec, dns_resolve, etc.)
   - Columns: Sources (collectors + integrations)
   - Cells: Fidelity indicator (🟢 HARD / 🟡 SOFT / ⚫ None)
   - Toggle to show/hide collectors

3. **Join Key Support Matrix**
   - Rows: Join key types
   - Columns: Sources
   - Cells: ✅ supported / — not supported

4. **Sample Event Inspector**
   - Modal showing raw → mapped event transformation
   - Provenance badges for derived scope keys
   - Join confidence percentages

### Incident Detail Additions

When viewing an incident, provenance information is surfaced:

- **EQS (Evidence Quality Score)** - Overall evidence quality
- **Provenance badges** - HARD vs SOFT fidelity per claim
- **Join confidence** - How confident we are in entity linkage
- **Missing-link warnings** - Alerts when key joins are missing

## Health Status Logic

Health status is computed automatically:

```rust
fn compute_health(&self) -> HealthStatus {
    // Stale check (>5 minutes since last event)
    if let Some(last_seen) = self.last_seen_ts {
        if Utc::now() - last_seen > Duration::minutes(5) {
            return HealthStatus::Warning;
        }
    } else {
        return HealthStatus::Unknown;
    }

    // Error rate check (>10% = warning, >50% = error)
    if self.parse_error_rate > 0.5 {
        return HealthStatus::Error;
    } else if self.parse_error_rate > 0.1 {
        return HealthStatus::Warning;
    }

    HealthStatus::Healthy
}
```

## Mapping Provenance

Every mapped event tracks its provenance:

```rust
pub struct MappedEvent {
    pub integration_id: String,
    pub raw_event_id: Option<String>,
    pub raw_json_hash: String,        // SHA256 of raw JSON
    pub mapping_version: String,
    pub derived_scope_keys: Vec<DerivedScopeKey>,
    pub mapped_at: DateTime<Utc>,
}

pub struct DerivedScopeKey {
    pub key: String,
    pub key_type: String,            // "process", "file", "socket", etc.
    pub fidelity: Fidelity,
    pub join_confidence: f64,        // 0.0 - 1.0
    pub confidence_reason: Option<String>,
}
```

This enables:
- **Auditing** - Trace any fact back to raw source
- **Reproducibility** - Same raw event + version = same mapping
- **Confidence scoring** - Understand reliability of evidence

## Factory Methods

Pre-configured profiles for common integrations:

```rust
// Create a Wazuh profile
let wazuh = IntegrationProfile::wazuh("wazuh_main".to_string());

// Create a Zeek profile
let zeek = IntegrationProfile::zeek("zeek_network".to_string());

// Create an Osquery profile
let osquery = IntegrationProfile::osquery("osquery_fleet".to_string());

// Create a JSONL export profile
let export = IntegrationProfile::jsonl_export("jsonl_export".to_string());
```

## Acceptance Tests

The following tests verify the system:

1. **Profile List + Update** - `/api/integrations` returns profiles, updates reflect
2. **Capabilities Matrix** - Merged view with collectors + integrations
3. **Sample Events** - Raw → mapped with provenance
4. **Health Status** - Automatic computation based on staleness/errors
5. **Mode Filtering** - Filter by export/ingest/both
6. **404 Handling** - Non-existent integrations return 404
7. **Collector Toggle** - Can exclude collectors from matrix
8. **Hash Determinism** - Same raw event = same hash

Run tests:
```bash
cargo test --test integration_profiles_test
```

## Files

| File | Purpose |
|------|---------|
| `crates/locald/src/integrations/profile.rs` | IntegrationProfile model |
| `crates/server/src/integration_api.rs` | REST API endpoints |
| `ui/index.html` | Integrations page HTML |
| `ui/app.js` | Integrations page JavaScript |
| `tests/integration_profiles_test.rs` | Acceptance tests |
| `docs/integrations_ui.md` | This documentation |
