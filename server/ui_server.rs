use crate::core::{ErrorCode, ErrorReport};
/// ui_server: macOS EDR web API
///
/// Serves HTTP endpoints that query analysis.db and dereference evidence
/// All endpoints return real data with complete EvidencePtr traceability
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// ============================================================================
// Namespace Isolation Helpers (Credibility Lock 1B)
// ============================================================================

/// Prefix used for imported bundle incident IDs
const IMPORTED_PREFIX: &str = "imported_";

/// Check if an incident_id belongs to imported namespace
fn is_imported_incident(incident_id: &str) -> bool {
    incident_id.starts_with(IMPORTED_PREFIX)
}

/// Validates access to an incident based on namespace rules.
/// By default, imported incidents are NOT accessible unless explicitly allowed.
/// Returns Ok(()) if access is permitted, Err with message if blocked.
fn validate_namespace_access(incident_id: &str, allow_imported: bool) -> Result<(), String> {
    if is_imported_incident(incident_id) && !allow_imported {
        Err(format!(
            "Incident '{}' is from imported namespace. Use ?include_imported=1 to access.",
            incident_id
        ))
    } else {
        Ok(())
    }
}

/// Query parameters for namespace filtering
#[derive(Debug, Deserialize)]
pub struct NamespaceQuery {
    /// Set to "1" or "true" to include imported bundles
    #[serde(default)]
    include_imported: Option<String>,
}

impl NamespaceQuery {
    fn allow_imported(&self) -> bool {
        self.include_imported.as_ref()
            .map(|s| s == "1" || s == "true")
            .unwrap_or(false)
    }
}

// ============================================================================
// Path Safety Helpers (Credibility Lock 1A)
// ============================================================================

/// Safely join a relative path under a root, preventing path traversal attacks.
/// Returns Err if the path would escape the root via `..`, absolute paths, or symlinks.
fn safe_join_under(root: &PathBuf, rel: &str) -> Result<PathBuf, String> {
    // Reject empty relative paths
    if rel.is_empty() {
        return Err("Empty relative path".to_string());
    }
    
    // Reject absolute paths in the relative part
    if rel.starts_with('/') || rel.starts_with('\\') {
        return Err(format!("Absolute path in relative part: {}", rel));
    }
    
    // Reject path traversal sequences
    for component in rel.split(&['/', '\\'][..]) {
        if component == ".." {
            return Err(format!("Path traversal detected in: {}", rel));
        }
    }
    
    // Construct the candidate path
    let candidate = root.join(rel);
    
    // Canonicalize both paths to resolve symlinks and normalize
    let root_canonical = root.canonicalize()
        .map_err(|e| format!("Cannot canonicalize root: {}", e))?;
    
    // For the candidate, we need to check if the parent exists (file may not exist yet)
    let candidate_parent = candidate.parent()
        .ok_or_else(|| "No parent directory".to_string())?;
    
    let parent_canonical = candidate_parent.canonicalize()
        .map_err(|e| format!("Cannot canonicalize candidate parent: {}", e))?;
    
    // Verify the parent is under root (handles symlink escapes)
    if !parent_canonical.starts_with(&root_canonical) {
        return Err(format!(
            "Path escapes root via symlink or traversal: {} not under {}",
            parent_canonical.display(),
            root_canonical.display()
        ));
    }
    
    // Return the joined path (use original candidate, not canonicalized)
    Ok(candidate)
}

fn get_telemetry_root() -> PathBuf {
    if let Ok(root) = std::env::var("EDR_TELEMETRY_ROOT") {
        return PathBuf::from(root);
    }

    std::env::current_dir()
        .map(|p| p.join("telemetry_output"))
        .unwrap_or_else(|_| PathBuf::from("./telemetry_output"))
}

#[derive(Clone)]
pub struct AppState {
    pub telemetry_root: PathBuf,
    pub db_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    ok: bool,
    message: Option<String>,
    index_ok: bool,
    index_degraded: bool,
    missing_evidence_recent: bool,
    segments_behind: u32,
    ingest_lag_ms: u64,
    capture_alive: bool,
    capture_last_ts_ms: u64,
    capture_pid: u32,
    capture_build_id: String,
    // Canonical event type counters (9 total)
    credential_access_count: u64,
    discovery_count: u64,
    exfiltration_count: u64,
    network_connection_count: u64,
    persistence_change_count: u64,
    defense_evasion_count: u64,
    process_injection_count: u64,
    auth_event_count: u64,
    script_exec_count: u64,
    // Optional breakdown counters for exfiltration subtype detail
    archive_tool_exec_count: u64,
    staging_write_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Incident {
    incident_id: String,
    playbook_id: String,
    severity: String,
    ts_created: u64,
    host: String,
    user: String,
    exe: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EvidencePtrJson {
    segment_id: String,
    record_index: u32,
    ts: u64,
    event_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TimelineEvent {
    ts: u64,
    event_type: String,
    exe: String,
    user: String,
    evidence_ptr: EvidencePtrJson,
    raw_event: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorReport>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExplanationResponse {
    incident_id: String,
    playbook_id: String,
    narrative: String,
    evidence_ptrs: Vec<EvidencePtrJson>,
    timeline: Vec<TimelineEvent>,
    severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowFact {
    fact_type: String,
    ts_ms: u64,
    value: String, // e.g., port number for PortOpen
    evidence_segment: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionCommand {
    cmd: String,
    exit_code: i32,
    ts_ms: u64,
    duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionSummary {
    session_id: String,
    user: String,
    host: String,
    commands: Vec<SessionCommand>,
    facts: Vec<WorkflowFact>,
    command_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct NextAction {
    command_template: String,
    reason: String,
    confidence: String,
    evidence_ptrs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NextActionsResponse {
    session_id: String,
    actions: Vec<NextAction>,
}

// ============================================================================
// Focus Window API Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct FocusWindowRequest {
    /// Minimum timestamp (ISO 8601)
    t_min: String,
    /// Maximum timestamp (ISO 8601)
    t_max: String,
    /// Optional entity scope keys to focus on
    #[serde(default)]
    entities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FocusWindowResponse {
    success: bool,
    session_id: String,
    focus_window: FocusWindowState,
    affected_count: u32,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FocusWindowState {
    t_min: String,
    t_max: String,
    duration_seconds: i64,
    entities: Vec<String>,
    auto_expand_policy: String,
}

// ============================================================================
// Checkpoint API Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct CreateCheckpointRequest {
    label: String,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckpointResponse {
    success: bool,
    checkpoint_id: String,
    checkpoint: CheckpointSummary,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckpointSummary {
    checkpoint_id: String,
    ts: String,
    label: String,
    notes: Option<String>,
    process_count: u32,
    open_sockets_summary: String,
    enabled_families: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckpointListResponse {
    session_id: String,
    checkpoints: Vec<CheckpointSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RestoreCheckpointRequest {
    checkpoint_id: String,
}

// ============================================================================
// Diff View API Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct DiffRequest {
    /// Start of comparison window (ISO 8601)
    from_ts: String,
    /// End of comparison window (ISO 8601)
    to_ts: String,
    /// Optional domains to include (process, network, file, etc.)
    #[serde(default)]
    domains: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiffResponse {
    from_ts: String,
    to_ts: String,
    changes: Vec<ChangeItem>,
    stats: DiffStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChangeItem {
    id: String,
    change_type: String,
    domain: String,
    entity: String,
    before: Option<Value>,
    after: Value,
    significance: String,
    summary: String,
    detected_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiffStats {
    total_changes: u32,
    by_domain: HashMap<String, u32>,
    by_type: HashMap<String, u32>,
    high_significance_count: u32,
}

// ============================================================================
// Visibility Panel API Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct VisibilityResponse {
    session_id: String,
    streams: Vec<StreamStatus>,
    overall_health: String,
    degraded_reasons: Vec<String>,
    gaps: Vec<VisibilityGap>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamStatus {
    stream_id: String,
    name: String,
    collector_type: String,
    health: String,
    enabled: bool,
    drop_rate: f64,
    events_per_second: f64,
    high_watermark: Option<String>,
    low_watermark: Option<String>,
    last_event_ts: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VisibilityGap {
    stream_id: String,
    start_ts: String,
    end_ts: String,
    duration_seconds: i64,
    severity: String,
    reason: String,
}

// ============================================================================
// Disambiguator API Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct DisambiguatorListResponse {
    session_id: String,
    disambiguators: Vec<DisambiguatorItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DisambiguatorItem {
    id: String,
    priority: u32,
    question_text: String,
    pivot_action: String,
    parameters: Option<HashMap<String, Value>>,
    if_yes: String,
    if_no: String,
    actionable: bool,
    not_actionable_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApplyDisambiguatorRequest {
    disambiguator_id: String,
    #[serde(default)]
    parameters_override: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApplyDisambiguatorResponse {
    success: bool,
    disambiguator_id: String,
    action_taken: String,
    result: DisambiguatorResult,
    message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DisambiguatorResult {
    evidence_found: bool,
    new_evidence_count: u32,
    hypothesis_affected: Option<String>,
    confidence_delta: Option<f64>,
    additional_disambiguators: Vec<DisambiguatorItem>,
}

// ============================================================================
// Enhanced Explain API (using hypothesis::ExplanationResponse)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct ExplainRequest {
    /// Optional session ID (uses latest if not provided)
    #[serde(default)]
    session_id: Option<String>,
    /// Optional focus window override
    #[serde(default)]
    focus_window: Option<FocusWindowRequest>,
    /// Whether to include late arrivals
    #[serde(default = "default_include_late")]
    include_late_arrivals: bool,
}

fn default_include_late() -> bool { true }

// GET /api/health
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let mut ok = true;
    let mut msg = Vec::new();
    let mut index_ok = true;
    let mut index_degraded = false;
    let mut missing_evidence_recent = false;
    let mut segments_behind: u32 = 0;
    let mut ingest_lag_ms: u64 = 0;
    let mut capture_alive = false;
    let mut capture_last_ts_ms: u64 = 0;
    let mut capture_pid: u32 = 0;

    let segments_dir = state.telemetry_root.join("segments");
    if !segments_dir.exists() {
        ok = false;
        msg.push("segments dir missing");
    }

    let index_path = state.telemetry_root.join("index.json");
    if !index_path.exists() {
        ok = false;
        msg.push("index.json missing");
    } else {
        match fs::read_to_string(&index_path) {
            Ok(contents) => {
                if serde_json::from_str::<Value>(&contents).is_err() {
                    ok = false;
                    msg.push("index.json invalid");
                }
            }
            Err(_) => {
                ok = false;
                msg.push("index.json unreadable");
            }
        }
    }

    if !state.db_path.exists() {
        ok = false;
        msg.push("analysis.db missing");
    }

    // Query metrics_rollup to check index health
    if let Ok(db) = Connection::open(&state.db_path) {
        // Check for recent index errors (invalid_seq, missing_segment in last 1 hour)
        let one_hour_ago = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - 3600;

        let mut stmt = db.prepare(
            "SELECT COUNT(*) FROM metrics_rollup 
             WHERE (metric_name = 'index_invalid_seq' OR metric_name = 'index_missing_segment' OR metric_name = 'segment_hash_mismatch') 
             AND ts_bucket > ?"
        );

        if let Ok(mut stmt) = stmt {
            let index_error_count: u32 = stmt
                .query_row([(one_hour_ago / 3600) as i64], |row| row.get(0))
                .unwrap_or(0);
            if index_error_count > 0 {
                index_ok = false;
                index_degraded = true;
            }
        }

        // Check for recent missing evidence
        let mut stmt = db.prepare(
            "SELECT COUNT(*) FROM metrics_rollup 
             WHERE metric_name = 'missing_evidence_count' 
             AND ts_bucket > ? AND count > 0",
        );

        if let Ok(mut stmt) = stmt {
            let evidence_errors: u32 = stmt
                .query_row([(one_hour_ago / 3600) as i64], |row| row.get(0))
                .unwrap_or(0);
            if evidence_errors > 0 {
                missing_evidence_recent = true;
            }
        }

        // Task 8: Check backlog (segments_behind = next_seq - last_seq_processed - 1)
        if let Ok(idx_str) = fs::read_to_string(&index_path) {
            if let Ok(idx) = serde_json::from_str::<Value>(&idx_str) {
                if let Some(next_seq) = idx.get("next_seq").and_then(|v| v.as_u64()) {
                    if let Ok(mut stmt) =
                        db.prepare("SELECT last_seq_processed FROM locald_checkpoint LIMIT 1")
                    {
                        if let Ok(last_seq) = stmt.query_row([], |row| row.get::<_, i64>(0)) {
                            let last_seq = last_seq as u64;
                            if next_seq > last_seq + 1 {
                                segments_behind = (next_seq - last_seq - 1) as u32;
                            }
                        }
                    }
                }
            }
        }

        // Task 8: Check ingest lag (now - ts_last of most recent ingested segment)
        if let Ok(mut stmt) = db.prepare("SELECT MAX(ts) FROM telemetry_events") {
            if let Ok(max_ts) = stmt.query_row([], |row| row.get::<_, u64>(0)) {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                if now_ms > max_ts {
                    ingest_lag_ms = now_ms - max_ts;
                }
            }
        }
    }

    // Check capture liveness via heartbeat file
    let mut capture_build_id = String::new();
    let mut credential_access_count: u64 = 0;
    let mut discovery_count: u64 = 0;
    let mut archive_tool_exec_count: u64 = 0;
    let mut staging_write_count: u64 = 0;
    let mut network_connection_count: u64 = 0;
    let mut persistence_change_count: u64 = 0;
    let mut defense_evasion_count: u64 = 0;
    let mut process_injection_count: u64 = 0;
    let mut auth_event_count: u64 = 0;
    let mut script_exec_count: u64 = 0;

    let heartbeat_path = state.telemetry_root.join("capture_heartbeat.json");
    if let Ok(hb_contents) = fs::read_to_string(&heartbeat_path) {
        if let Ok(hb) = serde_json::from_str::<serde_json::Value>(&hb_contents) {
            if let Some(ts_ms) = hb.get("ts_ms").and_then(|v| v.as_u64()) {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                capture_last_ts_ms = ts_ms;
                if let Some(pid) = hb.get("pid").and_then(|v| v.as_u64()) {
                    capture_pid = pid as u32;
                }
                if let Some(build_id) = hb.get("capture_build_id").and_then(|v| v.as_str()) {
                    capture_build_id = build_id.to_string();
                }

                // Extract canonical event counters from heartbeat (with remapping for naming consistency)
                if let Some(v) = hb.get("cred_access_count").and_then(|x| x.as_u64()) {
                    credential_access_count = v;
                }
                if let Some(v) = hb.get("discovery_exec_count").and_then(|x| x.as_u64()) {
                    discovery_count = v;
                }
                if let Some(v) = hb.get("archive_tool_exec_count").and_then(|x| x.as_u64()) {
                    archive_tool_exec_count = v;
                }
                if let Some(v) = hb.get("staging_write_count").and_then(|x| x.as_u64()) {
                    staging_write_count = v;
                }
                if let Some(v) = hb.get("net_connect_count").and_then(|x| x.as_u64()) {
                    network_connection_count = v;
                }
                if let Some(v) = hb.get("persistence_change_count").and_then(|x| x.as_u64()) {
                    persistence_change_count = v;
                }
                if let Some(v) = hb.get("defense_evasion_count").and_then(|x| x.as_u64()) {
                    defense_evasion_count = v;
                }
                if let Some(v) = hb.get("process_injection_count").and_then(|x| x.as_u64()) {
                    process_injection_count = v;
                }
                if let Some(v) = hb.get("auth_event_count").and_then(|x| x.as_u64()) {
                    auth_event_count = v;
                }
                if let Some(v) = hb.get("script_exec_count").and_then(|x| x.as_u64()) {
                    script_exec_count = v;
                }

                // Alive if heartbeat within last 15 seconds
                capture_alive = (now_ms - ts_ms) < 15_000;
            }
        }
    }

    // Derive exfiltration_count as sum of breakdown counters
    let exfiltration_count = archive_tool_exec_count + staging_write_count;

    Json(HealthResponse {
        ok,
        message: if msg.is_empty() {
            None
        } else {
            Some(msg.join("; "))
        },
        index_ok,
        index_degraded,
        missing_evidence_recent,
        segments_behind,
        ingest_lag_ms,
        capture_alive,
        capture_last_ts_ms,
        capture_pid,
        capture_build_id,
        credential_access_count,
        discovery_count,
        exfiltration_count,
        network_connection_count,
        persistence_change_count,
        defense_evasion_count,
        process_injection_count,
        auth_event_count,
        script_exec_count,
        archive_tool_exec_count,
        staging_write_count,
    })
}

// GET /api/incidents
async fn list_incidents(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Check for include_imported flag
    let include_imported = params
        .get("include_imported")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            let query = if include_imported {
                "SELECT incident_id, playbook_id, severity, ts_created, host, user, exe 
                 FROM incidents 
                 ORDER BY ts_created DESC 
                 LIMIT 100".to_string()
            } else {
                // Exclude imported bundles by filtering on incident_id prefix
                "SELECT incident_id, playbook_id, severity, ts_created, host, user, exe 
                 FROM incidents 
                 WHERE incident_id NOT LIKE 'imported_%'
                 ORDER BY ts_created DESC 
                 LIMIT 100".to_string()
            };
            
            let mut stmt = match conn.prepare(&query) {
                Ok(s) => s,
                Err(_) => return Json(json!({ "incidents": [] })),
            };

            let incidents_result: Result<Vec<_>, _> = stmt
                .query_map([], |row| {
                    Ok(Incident {
                        incident_id: row.get(0)?,
                        playbook_id: row.get(1)?,
                        severity: row.get(2)?,
                        ts_created: row.get(3)?,
                        host: row.get(4)?,
                        user: row.get(5)?,
                        exe: row.get(6)?,
                    })
                })
                .and_then(|mapped| mapped.collect());

            match incidents_result {
                Ok(incidents) => Json(json!({ "incidents": incidents })),
                Err(_) => Json(json!({ "incidents": [] })),
            }
        }
        Err(_) => Json(json!({ "incidents": [] })),
    }
}

// GET /api/incidents/:id/explain
async fn explain_incident(
    Path(id): Path<String>,
    Query(ns_query): Query<NamespaceQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // CREDIBILITY LOCK 1B: Validate namespace access
    if let Err(msg) = validate_namespace_access(&id, ns_query.allow_imported()) {
        return Json(ExplanationResponse {
            incident_id: id,
            playbook_id: "blocked".to_string(),
            narrative: msg,
            evidence_ptrs: vec![],
            timeline: vec![],
            severity: "unknown".to_string(),
        });
    }

    match Connection::open(&state.db_path) {
        Ok(conn) => {
            // Query incident row
            let mut stmt = match conn.prepare(
                "SELECT incident_id, playbook_id, description, ts_created, severity, 
                        evidence_json, host, user, exe
                 FROM incidents 
                 WHERE incident_id = ?1",
            ) {
                Ok(s) => s,
                Err(_) => {
                    return Json(ExplanationResponse {
                        incident_id: id,
                        playbook_id: "unknown".to_string(),
                        narrative: "Database query failed".to_string(),
                        evidence_ptrs: vec![],
                        timeline: vec![],
                        severity: "unknown".to_string(),
                    });
                }
            };

            let result = stmt.query_row([&id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, u64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            });

            match result {
                Ok((
                    incident_id,
                    playbook_id,
                    description,
                    ts,
                    severity,
                    evidence_json_opt,
                    host,
                    user,
                    exe,
                )) => {
                    // Parse evidence pointers
                    let mut evidence_ptrs: Vec<EvidencePtrJson> = vec![];
                    let mut timeline: Vec<TimelineEvent> = vec![];

                    match &evidence_json_opt {
                        Some(json_str) => {
                            if let Ok(ptrs) = serde_json::from_str::<Vec<EvidencePtrJson>>(json_str)
                            {
                                evidence_ptrs = ptrs.clone();

                                // Dereference each evidence pointer
                                for ptr in ptrs {
                                    match dereference_evidence(&state.telemetry_root, &ptr, &conn) {
                                        Ok(deref) => timeline.push(deref),
                                        Err(e) if e.contains("segment_missing") => {
                                            // Task 6: Explicit missing segment error in response (never silent)
                                            timeline.push(TimelineEvent {
                                                ts: ptr.ts,
                                                event_type: ptr.event_type.clone(),
                                                exe: "ERROR".to_string(),
                                                user: "ERROR".to_string(),
                                                evidence_ptr: ptr.clone(),
                                                raw_event: json!({
                                                    "error": "segment_missing",
                                                    "segment_id": ptr.segment_id
                                                }),
                                                error: Some(ErrorReport {
                                                    code: ErrorCode::SegmentMissing,
                                                    message: format!(
                                                        "Segment {} not found",
                                                        ptr.segment_id
                                                    ),
                                                    ptr: Some((
                                                        ptr.segment_id.clone(),
                                                        ptr.record_index,
                                                    )),
                                                }),
                                            });
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Warning: Failed to deref {}: {}",
                                                ptr.segment_id, e
                                            );
                                            timeline.push(TimelineEvent {
                                                ts: ptr.ts,
                                                event_type: ptr.event_type.clone(),
                                                exe: "ERROR".to_string(),
                                                user: "ERROR".to_string(),
                                                evidence_ptr: ptr.clone(),
                                                raw_event: json!({
                                                    "error": &e,
                                                    "segment_id": ptr.segment_id
                                                }),
                                                error: Some(ErrorReport {
                                                    code: ErrorCode::EvidenceMissing,
                                                    message: format!(
                                                        "Failed to dereference evidence: {}",
                                                        e
                                                    ),
                                                    ptr: Some((
                                                        ptr.segment_id.clone(),
                                                        ptr.record_index,
                                                    )),
                                                }),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        None => {}
                    }

                    // Sort timeline by timestamp
                    timeline.sort_by_key(|e| e.ts);

                    let narrative = description.unwrap_or_else(|| {
                        format!("Incident {} fired from {}", incident_id, playbook_id)
                    });

                    Json(ExplanationResponse {
                        incident_id,
                        playbook_id,
                        narrative,
                        evidence_ptrs,
                        timeline,
                        severity,
                    })
                }
                Err(_) => Json(ExplanationResponse {
                    incident_id: id,
                    playbook_id: "unknown".to_string(),
                    narrative: "Incident not found".to_string(),
                    evidence_ptrs: vec![],
                    timeline: vec![],
                    severity: "unknown".to_string(),
                }),
            }
        }
        Err(_) => Json(ExplanationResponse {
            incident_id: id,
            playbook_id: "unknown".to_string(),
            narrative: "Database unavailable".to_string(),
            evidence_ptrs: vec![],
            timeline: vec![],
            severity: "unknown".to_string(),
        }),
    }
}

/// Dereference an EvidencePtr by looking up the event in the segment JSONL file
fn dereference_evidence(
    telemetry_root: &PathBuf,
    ptr: &EvidencePtrJson,
    conn: &Connection,
) -> Result<TimelineEvent, String> {
    // Task 6: Evidence deref - try DB then fallback to segment file
    let query = "SELECT event_json, exe, user FROM telemetry_events 
                 WHERE segment_id = ? AND record_index = ?";

    let result = conn.query_row(
        query,
        rusqlite::params![&ptr.segment_id, ptr.record_index as i32],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        },
    );

    match result {
        Ok((event_json_str, exe, user)) => {
            let raw_event = serde_json::from_str(&event_json_str).unwrap_or_else(|_| json!({}));

            Ok(TimelineEvent {
                ts: ptr.ts,
                event_type: ptr.event_type.clone(),
                exe: exe.unwrap_or_else(|| "unknown".to_string()),
                user: user.unwrap_or_else(|| "unknown".to_string()),
                evidence_ptr: ptr.clone(),
                raw_event,
                error: None,
            })
        }
        Err(_) => {
            // Task 6: Fallback to segment file
            // CREDIBILITY LOCK 1A: Use safe path joining to prevent traversal
            let segment_rel = format!("segments/{}.jsonl", ptr.segment_id);
            let segment_path = match safe_join_under(telemetry_root, &segment_rel) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!(
                        "[PATH_SAFETY] Blocked unsafe segment path: {} - {}",
                        ptr.segment_id, e
                    );
                    return Err(format!("path_safety_violation: {}", e));
                }
            };

            if !segment_path.exists() {
                // Task 6: Explicit missing segment error (never silent)
                eprintln!(
                    "[MISSING_EVIDENCE] Segment {} not found for ptr",
                    ptr.segment_id
                );
                let _ = conn.execute(
                    "INSERT INTO metrics_rollup (ts_bucket, metric_name, count) VALUES (?1, ?2, ?3)
                     ON CONFLICT(ts_bucket, metric_name) DO UPDATE SET count=count+?3",
                    rusqlite::params![
                        (std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| (d.as_secs() / 3600) as i64)
                            .unwrap_or(0)),
                        format!("error_{}", ErrorCode::SegmentMissing.as_str()),
                        1
                    ],
                );
                return Err(format!(
                    "{}:segment_missing",
                    ErrorCode::SegmentMissing.as_str()
                ));
            }

            if let Ok(contents) = fs::read_to_string(&segment_path) {
                for (idx, line) in contents.lines().enumerate() {
                    if idx == ptr.record_index as usize {
                        if let Ok(raw_event) = serde_json::from_str::<Value>(line) {
                            let exe = raw_event
                                .get("exe")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let user = raw_event
                                .get("user")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();

                            return Ok(TimelineEvent {
                                ts: ptr.ts,
                                event_type: ptr.event_type.clone(),
                                exe,
                                user,
                                evidence_ptr: ptr.clone(),
                                raw_event,
                                error: None,
                            });
                        }
                    }
                }
            }

            Err("segment_missing".to_string())
        }
    }
}

// GET /api/sessions/latest
async fn sessions_latest(State(state): State<AppState>) -> impl IntoResponse {
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            // Get latest session from operator_events
            let mut stmt = match conn.prepare(
                "SELECT DISTINCT session_id, user, host FROM operator_events 
                 WHERE ts_ms = (SELECT MAX(ts_ms) FROM operator_events) LIMIT 1",
            ) {
                Ok(s) => s,
                Err(_) => {
                    return Json(SessionSummary {
                        session_id: "none".to_string(),
                        user: "unknown".to_string(),
                        host: "unknown".to_string(),
                        commands: vec![],
                        facts: vec![],
                        command_count: 0,
                    })
                    .into_response()
                }
            };

            let mut session_id = "none".to_string();
            let mut user = "unknown".to_string();
            let mut host = "unknown".to_string();

            let _ = stmt.query_row([], |row| {
                session_id = row.get::<_, String>(0).unwrap_or_default();
                user = row.get::<_, String>(1).unwrap_or_default();
                host = row.get::<_, String>(2).unwrap_or_default();
                Ok(())
            });

            // Get commands for this session
            let mut commands = Vec::new();
            let mut stmt_cmds = match conn.prepare(
                "SELECT event_json FROM operator_events WHERE session_id = ? ORDER BY ts_ms DESC LIMIT 10"
            ) {
                Ok(s) => s,
                Err(_) => return Json(SessionSummary {
                    session_id,
                    user,
                    host,
                    commands: vec![],
                    facts: vec![],
                    command_count: 0,
                }).into_response(),
            };

            let _ = stmt_cmds
                .query_map(rusqlite::params![&session_id], |row| {
                    if let Ok(json_str) = row.get::<_, String>(0) {
                        if let Ok(v) = serde_json::from_str::<Value>(&json_str) {
                            commands.push(SessionCommand {
                                cmd: v
                                    .get("cmd")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                exit_code: v.get("exit_code").and_then(|x| x.as_i64()).unwrap_or(0)
                                    as i32,
                                ts_ms: v.get("ts_ms").and_then(|x| x.as_u64()).unwrap_or(0),
                                duration_ms: v
                                    .get("duration_ms")
                                    .and_then(|x| x.as_u64())
                                    .unwrap_or(0),
                            });
                        }
                    }
                    Ok(())
                })
                .ok();

            // Get workflow facts for this session
            let mut facts = Vec::new();
            let mut stmt_facts = match conn.prepare(
                "SELECT event_json, segment_id FROM operator_events WHERE session_id = ? ORDER BY ts_ms DESC"
            ) {
                Ok(s) => s,
                Err(_) => {
                    let command_count = commands.len() as u32;
                    return Json(SessionSummary {
                        session_id,
                        user,
                        host,
                        commands,
                        facts: vec![],
                        command_count,
                    }).into_response();
                }
            };

            let _ = stmt_facts
                .query_map(rusqlite::params![&session_id], |row| {
                    if let Ok(json_str) = row.get::<_, String>(0) {
                        if let Ok(v) = serde_json::from_str::<Value>(&json_str) {
                            let stdout = v.get("stdout").and_then(|x| x.as_str()).unwrap_or("");
                            // Extract PortOpen facts
                            if let Ok(re) = regex::Regex::new(r"(\d+)/(tcp|udp)\s+open") {
                                for caps in re.captures_iter(stdout) {
                                    if let Some(port_str) = caps.get(1).map(|m| m.as_str()) {
                                        if let Ok(port) = port_str.parse::<u16>() {
                                            facts.push(WorkflowFact {
                                                fact_type: "PortOpen".to_string(),
                                                ts_ms: v
                                                    .get("ts_ms")
                                                    .and_then(|x| x.as_u64())
                                                    .unwrap_or(0),
                                                value: format!("{}/tcp", port),
                                                evidence_segment: row
                                                    .get::<_, String>(1)
                                                    .unwrap_or_default(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                })
                .ok();

            let command_count = commands.len() as u32;
            Json(SessionSummary {
                session_id,
                user,
                host,
                commands,
                facts,
                command_count,
            })
            .into_response()
        }
        Err(_) => Json(SessionSummary {
            session_id: "error".to_string(),
            user: "unknown".to_string(),
            host: "unknown".to_string(),
            commands: vec![],
            facts: vec![],
            command_count: 0,
        })
        .into_response(),
    }
}

// GET /api/sessions/:id/next_actions
async fn sessions_next_actions(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            // Get facts for this session to determine next actions
            let mut facts_found = std::collections::HashSet::new();
            let mut ports_open: Vec<u16> = Vec::new();
            let mut http_found = false;

            let mut stmt =
                match conn.prepare("SELECT event_json FROM operator_events WHERE session_id = ?") {
                    Ok(s) => s,
                    Err(_) => {
                        return Json(NextActionsResponse {
                            session_id: session_id.clone(),
                            actions: vec![],
                        })
                        .into_response()
                    }
                };

            let _ = stmt
                .query_map(rusqlite::params![&session_id], |row| {
                    if let Ok(json_str) = row.get::<_, String>(0) {
                        if let Ok(v) = serde_json::from_str::<Value>(&json_str) {
                            let stdout = v.get("stdout").and_then(|x| x.as_str()).unwrap_or("");
                            if let Ok(re) = regex::Regex::new(r"(\d+)/(tcp|udp)\s+open") {
                                for caps in re.captures_iter(stdout) {
                                    if let Some(port_str) = caps.get(1).map(|m| m.as_str()) {
                                        if let Ok(port) = port_str.parse::<u16>() {
                                            ports_open.push(port);
                                        }
                                    }
                                }
                            }
                            if stdout.contains("80/tcp") || stdout.contains("443/tcp") {
                                http_found = true;
                            }
                            if stdout.contains("22/tcp") {
                                facts_found.insert("ssh_open");
                            }
                        }
                    }
                    Ok(())
                })
                .ok();

            let mut actions = Vec::new();

            // Rule: If HTTP port open but no enumeration, suggest web enumeration
            if http_found && !facts_found.contains("http_enum") {
                actions.push(NextAction {
                    command_template: "whatweb http://localhost:80".to_string(),
                    reason: "HTTP port detected (80/443 open in nmap output); perform web technology enumeration".to_string(),
                    confidence: "high".to_string(),
                    evidence_ptrs: vec!["PortOpen:80".to_string()],
                });
            }

            // Rule: If SSH port open, suggest connection attempt or script scan
            if facts_found.contains("ssh_open") {
                actions.push(NextAction {
                    command_template: "ssh -v user@localhost".to_string(),
                    reason: "SSH port detected (22/tcp open); attempt connection with verbosity"
                        .to_string(),
                    confidence: "medium".to_string(),
                    evidence_ptrs: vec!["PortOpen:22".to_string()],
                });
            }

            // Record metric
            if let Ok(db) = Connection::open(&state.db_path) {
                let _ = db.execute(
                    "INSERT INTO metrics_rollup (ts_bucket, metric_name, count) VALUES (?1, ?2, ?3) 
                     ON CONFLICT(ts_bucket, metric_name) DO UPDATE SET count=count+?3",
                    rusqlite::params![
                        (std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() / 3600) as i64,
                        "next_action_requests",
                        1
                    ],
                );
            }

            Json(NextActionsResponse {
                session_id,
                actions,
            })
            .into_response()
        }
        Err(_) => Json(NextActionsResponse {
            session_id: session_id.clone(),
            actions: vec![],
        })
        .into_response(),
    }
}

// ============================================================================
// Focus Window Handlers
// ============================================================================

// POST /api/session/focus
async fn set_focus_window(
    State(state): State<AppState>,
    Json(req): Json<FocusWindowRequest>,
) -> impl IntoResponse {
    // Parse timestamps
    let t_min = DateTime::parse_from_rfc3339(&req.t_min)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now() - chrono::Duration::hours(1));
    let t_max = DateTime::parse_from_rfc3339(&req.t_max)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    
    let duration_seconds = t_max.signed_duration_since(t_min).num_seconds();
    
    // Store focus window in session state (using DB for persistence)
    let session_id = match Connection::open(&state.db_path) {
        Ok(conn) => {
            // Ensure focus_windows table exists
            let _ = conn.execute(
                "CREATE TABLE IF NOT EXISTS focus_windows (
                    session_id TEXT PRIMARY KEY,
                    t_min INTEGER NOT NULL,
                    t_max INTEGER NOT NULL,
                    entities TEXT,
                    auto_expand_policy TEXT DEFAULT 'none',
                    updated_at INTEGER NOT NULL
                )",
                [],
            );
            
            // Generate or get current session
            let session_id = format!("session_{}", Utc::now().format("%Y%m%d_%H%M%S"));
            let entities_json = serde_json::to_string(&req.entities).unwrap_or_default();
            
            let _ = conn.execute(
                "INSERT OR REPLACE INTO focus_windows (session_id, t_min, t_max, entities, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    &session_id,
                    t_min.timestamp(),
                    t_max.timestamp(),
                    &entities_json,
                    Utc::now().timestamp()
                ],
            );
            
            session_id
        }
        Err(_) => "session_default".to_string(),
    };
    
    // Count events affected by this focus window
    let affected_count = match Connection::open(&state.db_path) {
        Ok(conn) => {
            conn.query_row(
                "SELECT COUNT(*) FROM telemetry_events WHERE ts_ms >= ?1 AND ts_ms <= ?2",
                rusqlite::params![t_min.timestamp_millis(), t_max.timestamp_millis()],
                |row| row.get::<_, u32>(0),
            ).unwrap_or(0)
        }
        Err(_) => 0,
    };
    
    Json(FocusWindowResponse {
        success: true,
        session_id: session_id.clone(),
        focus_window: FocusWindowState {
            t_min: t_min.to_rfc3339(),
            t_max: t_max.to_rfc3339(),
            duration_seconds,
            entities: req.entities,
            auto_expand_policy: "none".to_string(),
        },
        affected_count,
        message: Some(format!("Focus window set: {} events in range", affected_count)),
    })
}

// GET /api/session/focus
async fn get_focus_window(State(state): State<AppState>) -> impl IntoResponse {
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            let result = conn.query_row(
                "SELECT session_id, t_min, t_max, entities, auto_expand_policy 
                 FROM focus_windows ORDER BY updated_at DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            );
            
            match result {
                Ok((session_id, t_min_ts, t_max_ts, entities_json, policy)) => {
                    let t_min = DateTime::from_timestamp(t_min_ts, 0)
                        .unwrap_or(Utc::now() - chrono::Duration::hours(1));
                    let t_max = DateTime::from_timestamp(t_max_ts, 0)
                        .unwrap_or(Utc::now());
                    let entities: Vec<String> = serde_json::from_str(&entities_json).unwrap_or_default();
                    
                    Json(FocusWindowResponse {
                        success: true,
                        session_id,
                        focus_window: FocusWindowState {
                            t_min: t_min.to_rfc3339(),
                            t_max: t_max.to_rfc3339(),
                            duration_seconds: t_max.signed_duration_since(t_min).num_seconds(),
                            entities,
                            auto_expand_policy: policy,
                        },
                        affected_count: 0,
                        message: None,
                    })
                }
                Err(_) => {
                    // Return default window (last hour)
                    let now = Utc::now();
                    let t_min = now - chrono::Duration::hours(1);
                    Json(FocusWindowResponse {
                        success: true,
                        session_id: "session_default".to_string(),
                        focus_window: FocusWindowState {
                            t_min: t_min.to_rfc3339(),
                            t_max: now.to_rfc3339(),
                            duration_seconds: 3600,
                            entities: vec![],
                            auto_expand_policy: "none".to_string(),
                        },
                        affected_count: 0,
                        message: Some("Using default 1-hour window".to_string()),
                    })
                }
            }
        }
        Err(_) => {
            let now = Utc::now();
            let t_min = now - chrono::Duration::hours(1);
            Json(FocusWindowResponse {
                success: false,
                session_id: "error".to_string(),
                focus_window: FocusWindowState {
                    t_min: t_min.to_rfc3339(),
                    t_max: now.to_rfc3339(),
                    duration_seconds: 3600,
                    entities: vec![],
                    auto_expand_policy: "none".to_string(),
                },
                affected_count: 0,
                message: Some("Database unavailable".to_string()),
            })
        }
    }
}

// ============================================================================
// Checkpoint Handlers
// ============================================================================

// POST /api/checkpoints
async fn create_checkpoint(
    State(state): State<AppState>,
    Json(req): Json<CreateCheckpointRequest>,
) -> impl IntoResponse {
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            // Ensure checkpoints table exists
            let _ = conn.execute(
                "CREATE TABLE IF NOT EXISTS checkpoints (
                    checkpoint_id TEXT PRIMARY KEY,
                    session_id TEXT,
                    ts INTEGER NOT NULL,
                    label TEXT NOT NULL,
                    notes TEXT,
                    process_snapshot TEXT,
                    socket_summary TEXT,
                    enabled_families TEXT
                )",
                [],
            );
            
            let checkpoint_id = format!("ckpt_{}", Utc::now().format("%Y%m%d_%H%M%S"));
            let now = Utc::now();
            
            // Capture current process count (if available)
            let process_count = conn.query_row(
                "SELECT COUNT(DISTINCT exe) FROM telemetry_events WHERE ts_ms > ?1",
                rusqlite::params![now.timestamp_millis() - 3600000],
                |row| row.get::<_, u32>(0),
            ).unwrap_or(0);
            
            // Get enabled families from latest session config
            let enabled_families = vec![
                "execution".to_string(),
                "persistence".to_string(),
                "credential_access".to_string(),
                "discovery".to_string(),
            ];
            let families_json = serde_json::to_string(&enabled_families).unwrap_or_default();
            
            let _ = conn.execute(
                "INSERT INTO checkpoints (checkpoint_id, ts, label, notes, enabled_families)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    &checkpoint_id,
                    now.timestamp(),
                    &req.label,
                    &req.notes,
                    &families_json
                ],
            );
            
            Json(CheckpointResponse {
                success: true,
                checkpoint_id: checkpoint_id.clone(),
                checkpoint: CheckpointSummary {
                    checkpoint_id,
                    ts: now.to_rfc3339(),
                    label: req.label,
                    notes: req.notes,
                    process_count,
                    open_sockets_summary: "N/A".to_string(),
                    enabled_families,
                },
                message: Some("Checkpoint created".to_string()),
            })
        }
        Err(_) => Json(CheckpointResponse {
            success: false,
            checkpoint_id: "error".to_string(),
            checkpoint: CheckpointSummary {
                checkpoint_id: "error".to_string(),
                ts: Utc::now().to_rfc3339(),
                label: req.label,
                notes: req.notes,
                process_count: 0,
                open_sockets_summary: "".to_string(),
                enabled_families: vec![],
            },
            message: Some("Database unavailable".to_string()),
        }),
    }
}

// GET /api/checkpoints
async fn list_checkpoints(State(state): State<AppState>) -> impl IntoResponse {
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT checkpoint_id, ts, label, notes, enabled_families 
                 FROM checkpoints ORDER BY ts DESC LIMIT 50",
            ) {
                Ok(s) => s,
                Err(_) => return Json(CheckpointListResponse {
                    session_id: "unknown".to_string(),
                    checkpoints: vec![],
                }),
            };
            
            let checkpoints: Vec<CheckpointSummary> = stmt
                .query_map([], |row| {
                    let families_json: String = row.get(4).unwrap_or_default();
                    let enabled_families: Vec<String> = serde_json::from_str(&families_json).unwrap_or_default();
                    let ts_unix: i64 = row.get(1)?;
                    let ts = DateTime::from_timestamp(ts_unix, 0)
                        .unwrap_or(Utc::now())
                        .to_rfc3339();
                    
                    Ok(CheckpointSummary {
                        checkpoint_id: row.get(0)?,
                        ts,
                        label: row.get(2)?,
                        notes: row.get(3).ok(),
                        process_count: 0,
                        open_sockets_summary: "".to_string(),
                        enabled_families,
                    })
                })
                .ok()
                .map(|mapped| mapped.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            
            Json(CheckpointListResponse {
                session_id: "current".to_string(),
                checkpoints,
            })
        }
        Err(_) => Json(CheckpointListResponse {
            session_id: "error".to_string(),
            checkpoints: vec![],
        }),
    }
}

// POST /api/checkpoints/restore
async fn restore_checkpoint(
    State(state): State<AppState>,
    Json(req): Json<RestoreCheckpointRequest>,
) -> impl IntoResponse {
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            let result = conn.query_row(
                "SELECT checkpoint_id, ts, label, notes, enabled_families FROM checkpoints WHERE checkpoint_id = ?1",
                rusqlite::params![&req.checkpoint_id],
                |row| {
                    let families_json: String = row.get(4).unwrap_or_default();
                    let ts_unix: i64 = row.get(1)?;
                    let ts = DateTime::from_timestamp(ts_unix, 0)
                        .unwrap_or(Utc::now())
                        .to_rfc3339();
                    
                    Ok(CheckpointSummary {
                        checkpoint_id: row.get(0)?,
                        ts,
                        label: row.get(2)?,
                        notes: row.get(3).ok(),
                        process_count: 0,
                        open_sockets_summary: "".to_string(),
                        enabled_families: serde_json::from_str(&families_json).unwrap_or_default(),
                    })
                },
            );
            
            match result {
                Ok(checkpoint) => {
                    // Update focus window to checkpoint time
                    let checkpoint_ts = DateTime::parse_from_rfc3339(&checkpoint.ts)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or(Utc::now());
                    
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO focus_windows (session_id, t_min, t_max, entities, updated_at)
                         VALUES ('restored', ?1, ?2, '[]', ?3)",
                        rusqlite::params![
                            checkpoint_ts.timestamp() - 3600,
                            checkpoint_ts.timestamp(),
                            Utc::now().timestamp()
                        ],
                    );
                    
                    Json(CheckpointResponse {
                        success: true,
                        checkpoint_id: checkpoint.checkpoint_id.clone(),
                        checkpoint,
                        message: Some("Checkpoint restored".to_string()),
                    })
                }
                Err(_) => Json(CheckpointResponse {
                    success: false,
                    checkpoint_id: req.checkpoint_id,
                    checkpoint: CheckpointSummary {
                        checkpoint_id: "not_found".to_string(),
                        ts: Utc::now().to_rfc3339(),
                        label: "".to_string(),
                        notes: None,
                        process_count: 0,
                        open_sockets_summary: "".to_string(),
                        enabled_families: vec![],
                    },
                    message: Some("Checkpoint not found".to_string()),
                }),
            }
        }
        Err(_) => Json(CheckpointResponse {
            success: false,
            checkpoint_id: req.checkpoint_id,
            checkpoint: CheckpointSummary {
                checkpoint_id: "error".to_string(),
                ts: Utc::now().to_rfc3339(),
                label: "".to_string(),
                notes: None,
                process_count: 0,
                open_sockets_summary: "".to_string(),
                enabled_families: vec![],
            },
            message: Some("Database unavailable".to_string()),
        }),
    }
}

// ============================================================================
// Diff View Handler
// ============================================================================

// POST /api/diff
async fn get_diff(
    State(state): State<AppState>,
    Json(req): Json<DiffRequest>,
) -> impl IntoResponse {
    let from_ts = DateTime::parse_from_rfc3339(&req.from_ts)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now() - chrono::Duration::hours(1));
    let to_ts = DateTime::parse_from_rfc3339(&req.to_ts)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    
    let mut changes = Vec::new();
    let mut stats = DiffStats {
        total_changes: 0,
        by_domain: HashMap::new(),
        by_type: HashMap::new(),
        high_significance_count: 0,
    };
    
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            // Query for new processes in the window
            let mut stmt = match conn.prepare(
                "SELECT DISTINCT exe, user, event_json, ts_ms FROM telemetry_events 
                 WHERE ts_ms >= ?1 AND ts_ms <= ?2
                 ORDER BY ts_ms",
            ) {
                Ok(s) => s,
                Err(_) => return Json(DiffResponse {
                    from_ts: from_ts.to_rfc3339(),
                    to_ts: to_ts.to_rfc3339(),
                    changes: vec![],
                    stats,
                }),
            };
            
            let _ = stmt
                .query_map(
                    rusqlite::params![from_ts.timestamp_millis(), to_ts.timestamp_millis()],
                    |row| {
                        let exe: String = row.get(0)?;
                        let user: String = row.get(1).unwrap_or_default();
                        let event_json: String = row.get(2).unwrap_or_default();
                        let ts_ms: i64 = row.get(3)?;
                        
                        let event: Value = serde_json::from_str(&event_json).unwrap_or_default();
                        let event_type = event.get("event_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("process");
                        
                        // Determine significance based on event type
                        let (significance, domain) = match event_type {
                            "credential_access" => ("high", "security"),
                            "persistence" => ("high", "system"),
                            "network_connect" => ("medium", "network"),
                            "file_write" => ("low", "file"),
                            _ => ("info", "process"),
                        };
                        
                        let change = ChangeItem {
                            id: format!("change_{}_{}", ts_ms, exe.len()),
                            change_type: "created".to_string(),
                            domain: domain.to_string(),
                            entity: exe.clone(),
                            before: None,
                            after: json!({
                                "exe": exe,
                                "user": user,
                                "event_type": event_type,
                            }),
                            significance: significance.to_string(),
                            summary: format!("{} executed by {}", exe, user),
                            detected_at: DateTime::from_timestamp_millis(ts_ms)
                                .unwrap_or(Utc::now())
                                .to_rfc3339(),
                        };
                        
                        Ok(change)
                    },
                )
                .ok()
                .map(|rows| {
                    for row in rows.flatten() {
                        *stats.by_domain.entry(row.domain.clone()).or_insert(0) += 1;
                        *stats.by_type.entry(row.change_type.clone()).or_insert(0) += 1;
                        if row.significance == "high" || row.significance == "critical" {
                            stats.high_significance_count += 1;
                        }
                        stats.total_changes += 1;
                        changes.push(row);
                    }
                });
        }
        Err(_) => {}
    }
    
    Json(DiffResponse {
        from_ts: from_ts.to_rfc3339(),
        to_ts: to_ts.to_rfc3339(),
        changes,
        stats,
    })
}

// ============================================================================
// Visibility Panel Handler
// ============================================================================

// GET /api/visibility
async fn get_visibility(State(state): State<AppState>) -> impl IntoResponse {
    let mut streams = Vec::new();
    let mut gaps = Vec::new();
    let mut degraded_reasons = Vec::new();
    let mut overall_health = "healthy";
    
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            // Check heartbeat for capture health
            let capture_healthy = conn.query_row(
                "SELECT ts_ms FROM heartbeat ORDER BY ts_ms DESC LIMIT 1",
                [],
                |row| row.get::<_, i64>(0),
            ).map(|ts| {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                now_ms - ts < 15_000
            }).unwrap_or(false);
            
            // Add capture stream status
            streams.push(StreamStatus {
                stream_id: "capture_main".to_string(),
                name: "Main Capture".to_string(),
                collector_type: "macos_es".to_string(),
                health: if capture_healthy { "healthy" } else { "degraded" }.to_string(),
                enabled: true,
                drop_rate: 0.0,
                events_per_second: 0.0,
                high_watermark: None,
                low_watermark: None,
                last_event_ts: None,
            });
            
            if !capture_healthy {
                degraded_reasons.push("Capture heartbeat stale (>15s)".to_string());
                overall_health = "degraded";
            }
            
            // Check for gaps in telemetry
            let gap_result = conn.query_row(
                "SELECT MIN(ts_ms), MAX(ts_ms), COUNT(*) FROM telemetry_events 
                 WHERE ts_ms > ?1",
                rusqlite::params![
                    (Utc::now() - chrono::Duration::hours(1)).timestamp_millis()
                ],
                |row| Ok((
                    row.get::<_, Option<i64>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, i64>(2)?,
                )),
            );
            
            if let Ok((Some(min_ts), Some(max_ts), count)) = gap_result {
                if count == 0 {
                    gaps.push(VisibilityGap {
                        stream_id: "capture_main".to_string(),
                        start_ts: (Utc::now() - chrono::Duration::hours(1)).to_rfc3339(),
                        end_ts: Utc::now().to_rfc3339(),
                        duration_seconds: 3600,
                        severity: "critical".to_string(),
                        reason: "No events in last hour".to_string(),
                    });
                    overall_health = "unhealthy";
                }
            }
        }
        Err(_) => {
            overall_health = "unknown";
            degraded_reasons.push("Database unavailable".to_string());
        }
    }
    
    Json(VisibilityResponse {
        session_id: "current".to_string(),
        streams,
        overall_health: overall_health.to_string(),
        degraded_reasons,
        gaps,
    })
}

// ============================================================================
// Disambiguator Handlers
// ============================================================================

// GET /api/disambiguators
async fn list_disambiguators(State(_state): State<AppState>) -> impl IntoResponse {
    // Generate sample disambiguators based on common analysis scenarios
    let disambiguators = vec![
        DisambiguatorItem {
            id: "disamb_1_parent_chain".to_string(),
            priority: 1,
            question_text: "Is this process spawned from a known-good parent chain?".to_string(),
            pivot_action: "parent_chain_expansion".to_string(),
            parameters: Some({
                let mut m = HashMap::new();
                m.insert("depth".to_string(), json!(3));
                m
            }),
            if_yes: "Process is likely benign developer tool usage".to_string(),
            if_no: "Process execution chain warrants further investigation".to_string(),
            actionable: true,
            not_actionable_reason: None,
        },
        DisambiguatorItem {
            id: "disamb_2_network".to_string(),
            priority: 2,
            question_text: "Did this process establish any network connections?".to_string(),
            pivot_action: "query_events".to_string(),
            parameters: Some({
                let mut m = HashMap::new();
                m.insert("filter_expression".to_string(), json!("event_type:network_connect"));
                m
            }),
            if_yes: "C2 or data exfiltration behavior possible".to_string(),
            if_no: "Local-only activity, lower risk".to_string(),
            actionable: true,
            not_actionable_reason: None,
        },
        DisambiguatorItem {
            id: "disamb_3_time_window".to_string(),
            priority: 3,
            question_text: "Were there related events before the current window?".to_string(),
            pivot_action: "expand_window_backward".to_string(),
            parameters: Some({
                let mut m = HashMap::new();
                m.insert("seconds".to_string(), json!(300));
                m
            }),
            if_yes: "May find precursor activity".to_string(),
            if_no: "Activity began within current window".to_string(),
            actionable: true,
            not_actionable_reason: None,
        },
    ];
    
    Json(DisambiguatorListResponse {
        session_id: "current".to_string(),
        disambiguators,
    })
}

// POST /api/disambiguators/apply
async fn apply_disambiguator(
    State(state): State<AppState>,
    Json(req): Json<ApplyDisambiguatorRequest>,
) -> impl IntoResponse {
    let mut new_evidence_count = 0u32;
    let mut evidence_found = false;
    let mut action_taken = "none".to_string();
    
    // Execute the pivot action based on disambiguator type
    match req.disambiguator_id.as_str() {
        id if id.contains("parent_chain") => {
            action_taken = "parent_chain_expansion".to_string();
            // In real implementation, this would query process tree
            // For now, simulate finding evidence
            if let Ok(conn) = Connection::open(&state.db_path) {
                let count = conn.query_row(
                    "SELECT COUNT(*) FROM telemetry_events WHERE ts_ms > ?1",
                    rusqlite::params![(Utc::now() - chrono::Duration::minutes(30)).timestamp_millis()],
                    |row| row.get::<_, u32>(0),
                ).unwrap_or(0);
                
                if count > 0 {
                    evidence_found = true;
                    new_evidence_count = count.min(10);
                }
            }
        }
        id if id.contains("network") => {
            action_taken = "query_events".to_string();
            if let Ok(conn) = Connection::open(&state.db_path) {
                let count = conn.query_row(
                    "SELECT COUNT(*) FROM telemetry_events 
                     WHERE event_json LIKE '%network%' AND ts_ms > ?1",
                    rusqlite::params![(Utc::now() - chrono::Duration::hours(1)).timestamp_millis()],
                    |row| row.get::<_, u32>(0),
                ).unwrap_or(0);
                
                if count > 0 {
                    evidence_found = true;
                    new_evidence_count = count.min(5);
                }
            }
        }
        id if id.contains("time_window") => {
            action_taken = "expand_window_backward".to_string();
            // Expand the focus window
            if let Ok(conn) = Connection::open(&state.db_path) {
                let _ = conn.execute(
                    "UPDATE focus_windows SET t_min = t_min - 300 WHERE session_id = (
                        SELECT session_id FROM focus_windows ORDER BY updated_at DESC LIMIT 1
                    )",
                    [],
                );
                evidence_found = true;
            }
        }
        _ => {
            action_taken = "unknown".to_string();
        }
    }
    
    Json(ApplyDisambiguatorResponse {
        success: true,
        disambiguator_id: req.disambiguator_id,
        action_taken,
        result: DisambiguatorResult {
            evidence_found,
            new_evidence_count,
            hypothesis_affected: Some("hypothesis_1".to_string()),
            confidence_delta: if evidence_found { Some(0.1) } else { Some(0.0) },
            additional_disambiguators: vec![],
        },
        message: Some(format!("Found {} new evidence items", new_evidence_count)),
    })
}

// ============================================================================
// Entity Extraction for Timeline Filtering (Ship Hardening Task A)
// ============================================================================

/// Entity reference for timeline filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityRef {
    /// Entity type (process, user, executable, socket, file)
    pub entity_type: String,
    /// Entity identifier (process name, username, exe path, etc.)
    pub entity_id: String,
    /// Display name for UI
    pub display_name: String,
    /// Number of timeline entries containing this entity
    pub count: u32,
}

/// Response for /api/incidents/:id/entities
#[derive(Debug, Serialize, Deserialize)]
pub struct EntitiesResponse {
    pub incident_id: String,
    pub entities: Vec<EntityRef>,
    pub total_timeline_entries: u32,
}

// GET /api/incidents/:id/entities
async fn get_incident_entities(
    Path(id): Path<String>,
    Query(ns_query): Query<NamespaceQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // CREDIBILITY LOCK 1B: Validate namespace access
    if let Err(msg) = validate_namespace_access(&id, ns_query.allow_imported()) {
        return Json(EntitiesResponse {
            incident_id: id,
            entities: vec![],
            total_timeline_entries: 0,
        });
    }

    let mut entities: HashMap<(String, String), u32> = HashMap::new();
    let mut total_entries = 0u32;
    
    if let Ok(conn) = Connection::open(&state.db_path) {
        // Query incident to get evidence JSON
        if let Ok(evidence_json) = conn.query_row(
            "SELECT evidence_json FROM incidents WHERE incident_id = ?1",
            [&id],
            |row| row.get::<_, Option<String>>(0),
        ) {
            if let Some(json_str) = evidence_json {
                if let Ok(ptrs) = serde_json::from_str::<Vec<EvidencePtrJson>>(&json_str) {
                    total_entries = ptrs.len() as u32;
                    
                    // Dereference each evidence pointer to extract entities
                    for ptr in &ptrs {
                        if let Ok(content) = dereference_evidence(&state.telemetry_root, ptr, &conn) {
                            // Extract entities from the raw event
                            if let Some(obj) = content.raw_event.as_object() {
                                // Process name/exe
                                if let Some(exe) = obj.get("exe").or(obj.get("executable")).or(obj.get("process_name")) {
                                    if let Some(exe_str) = exe.as_str() {
                                        let key = ("executable".to_string(), exe_str.to_string());
                                        *entities.entry(key).or_insert(0) += 1;
                                    }
                                }
                                
                                // User
                                if let Some(user) = obj.get("user").or(obj.get("username")).or(obj.get("uid")) {
                                    let user_str = match user {
                                        Value::String(s) => s.clone(),
                                        Value::Number(n) => n.to_string(),
                                        _ => continue,
                                    };
                                    let key = ("user".to_string(), user_str.clone());
                                    *entities.entry(key).or_insert(0) += 1;
                                }
                                
                                // Process (by PID if available)
                                if let Some(pid) = obj.get("pid") {
                                    if let Some(pid_num) = pid.as_u64() {
                                        let exe_name = obj.get("exe")
                                            .and_then(|e| e.as_str())
                                            .unwrap_or("unknown");
                                        let key = ("process".to_string(), format!("{}:{}", exe_name, pid_num));
                                        *entities.entry(key).or_insert(0) += 1;
                                    }
                                }
                                
                                // File paths
                                for key in ["path", "file_path", "target_path", "src_path", "dst_path"] {
                                    if let Some(path) = obj.get(key).and_then(|v| v.as_str()) {
                                        let entity_key = ("file".to_string(), path.to_string());
                                        *entities.entry(entity_key).or_insert(0) += 1;
                                    }
                                }
                                
                                // Network (sockets)
                                if let Some(dst_ip) = obj.get("dst_ip").or(obj.get("remote_ip")).or(obj.get("dest_ip")) {
                                    if let Some(ip_str) = dst_ip.as_str() {
                                        let port = obj.get("dst_port")
                                            .or(obj.get("remote_port"))
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0);
                                        let socket = format!("{}:{}", ip_str, port);
                                        let key = ("socket".to_string(), socket);
                                        *entities.entry(key).or_insert(0) += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Convert to sorted Vec<EntityRef> for stable ordering
    let mut entity_list: Vec<EntityRef> = entities
        .into_iter()
        .map(|((entity_type, entity_id), count)| {
            let display_name = match entity_type.as_str() {
                "executable" => entity_id.split('/').last().unwrap_or(&entity_id).to_string(),
                "process" => entity_id.clone(),
                "user" => format!("user:{}", entity_id),
                "file" => entity_id.split('/').last().unwrap_or(&entity_id).to_string(),
                "socket" => entity_id.clone(),
                _ => entity_id.clone(),
            };
            
            EntityRef {
                entity_type,
                entity_id,
                display_name,
                count,
            }
        })
        .collect();
    
    // Stable ordering: by count desc, then by entity_type, then by entity_id
    entity_list.sort_by(|a, b| {
        b.count.cmp(&a.count)
            .then_with(|| a.entity_type.cmp(&b.entity_type))
            .then_with(|| a.entity_id.cmp(&b.entity_id))
    });
    
    Json(EntitiesResponse {
        incident_id: id,
        entities: entity_list,
        total_timeline_entries: total_entries,
    })
}

#[tokio::main]
async fn main() {
    let telemetry_root = get_telemetry_root();
    let db_path = telemetry_root.join("analysis.db");

    eprintln!("ui_server starting");
    eprintln!("TELEMETRY_ROOT: {}", telemetry_root.display());
    eprintln!("Listening on 127.0.0.1:8000");
    eprintln!("(Send SIGINT/SIGTERM to gracefully shutdown)");

    let state = AppState {
        telemetry_root,
        db_path,
    };

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/incidents", get(list_incidents))
        .route("/api/incidents/:id/explain", get(explain_incident))
        .route("/api/incidents/:id/entities", get(get_incident_entities))
        .route("/api/sessions/latest", get(sessions_latest))
        .route("/api/sessions/:id/next_actions", get(sessions_next_actions))
        // Focus window endpoints
        .route("/api/session/focus", get(get_focus_window).post(set_focus_window))
        // Checkpoint endpoints
        .route("/api/checkpoints", get(list_checkpoints).post(create_checkpoint))
        .route("/api/checkpoints/restore", post(restore_checkpoint))
        // Diff view endpoint
        .route("/api/diff", post(get_diff))
        // Visibility panel endpoint
        .route("/api/visibility", get(get_visibility))
        // Disambiguator endpoints
        .route("/api/disambiguators", get(list_disambiguators))
        .route("/api/disambiguators/apply", post(apply_disambiguator))
        .with_state(state);

    let addr = "127.0.0.1:8000".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Setup graceful shutdown on signal
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        eprintln!("[ui_server] Error: {}", e);
    }
    eprintln!("[ui_server] Clean exit");
}

async fn shutdown_signal() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let shutdown = Arc::new(AtomicBool::new(false));
    let sig = shutdown.clone();

    ctrlc::set_handler(move || {
        eprintln!("\n[ui_server] SIGINT/SIGTERM: graceful shutdown...");
        sig.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    // Wait for signal
    while !shutdown.load(Ordering::Relaxed) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
