/// ui_server: macOS EDR web API
/// 
/// Serves HTTP endpoints that query analysis.db and dereference evidence
/// All endpoints return real data with complete EvidencePtr traceability

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use rusqlite::Connection;
use forensic_hooks::error_taxonomy::{ErrorCode, ErrorReport};

fn get_telemetry_root() -> PathBuf {
    if let Ok(root) = std::env::var("EDR_TELEMETRY_ROOT") {
        return PathBuf::from(root);
    }
    
    std::env::current_dir()
        .map(|p| p.join("telemetry_output"))
        .unwrap_or_else(|_| PathBuf::from("./telemetry_output"))
}

#[derive(Clone)]
struct AppState {
    telemetry_root: PathBuf,
    db_path: PathBuf,
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
    value: String,  // e.g., port number for PortOpen
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

// GET /api/health
async fn health(State(state): State<AppState>) -> impl IntoResponse {
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
            .as_secs() - 3600;
        
        let mut stmt = db.prepare(
            "SELECT COUNT(*) FROM metrics_rollup 
             WHERE (metric_name = 'index_invalid_seq' OR metric_name = 'index_missing_segment' OR metric_name = 'segment_hash_mismatch') 
             AND ts_bucket > ?"
        );
        
        if let Ok(mut stmt) = stmt {
            let index_error_count: u32 = stmt.query_row([(one_hour_ago / 3600) as i64], |row| row.get(0)).unwrap_or(0);
            if index_error_count > 0 {
                index_ok = false;
                index_degraded = true;
            }
        }
        
        // Check for recent missing evidence
        let mut stmt = db.prepare(
            "SELECT COUNT(*) FROM metrics_rollup 
             WHERE metric_name = 'missing_evidence_count' 
             AND ts_bucket > ? AND count > 0"
        );
        
        if let Ok(mut stmt) = stmt {
            let evidence_errors: u32 = stmt.query_row([(one_hour_ago / 3600) as i64], |row| row.get(0)).unwrap_or(0);
            if evidence_errors > 0 {
                missing_evidence_recent = true;
            }
        }
        
        // Task 8: Check backlog (segments_behind = next_seq - last_seq_processed - 1)
        if let Ok(idx_str) = fs::read_to_string(&index_path) {
            if let Ok(idx) = serde_json::from_str::<Value>(&idx_str) {
                if let Some(next_seq) = idx.get("next_seq").and_then(|v| v.as_u64()) {
                    if let Ok(mut stmt) = db.prepare(
                        "SELECT last_seq_processed FROM locald_checkpoint LIMIT 1"
                    ) {
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
        if let Ok(mut stmt) = db.prepare(
            "SELECT MAX(ts) FROM telemetry_events"
        ) {
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
                
                // Alive if heartbeat within last 15 seconds
                capture_alive = (now_ms - ts_ms) < 15_000;
            }
        }
    }
    
    Json(HealthResponse {
        ok,
        message: if msg.is_empty() { None } else { Some(msg.join("; ")) },
        index_ok,
        index_degraded,
        missing_evidence_recent,
        segments_behind,
        ingest_lag_ms,
        capture_alive,
        capture_last_ts_ms,
        capture_pid,
        capture_build_id,
    })
}

// GET /api/incidents
async fn list_incidents(State(state): State<AppState>) -> impl IntoResponse {
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT incident_id, playbook_id, severity, ts_created, host, user, exe 
                 FROM incidents 
                 ORDER BY ts_created DESC 
                 LIMIT 100"
            ) {
                Ok(s) => s,
                Err(_) => return Json(json!({ "incidents": [] })),
            };
            
            let incidents_result: Result<Vec<_>, _> = stmt.query_map([], |row| {
                Ok(Incident {
                    incident_id: row.get(0)?,
                    playbook_id: row.get(1)?,
                    severity: row.get(2)?,
                    ts_created: row.get(3)?,
                    host: row.get(4)?,
                    user: row.get(5)?,
                    exe: row.get(6)?,
                })
            }).and_then(|mapped| mapped.collect());
            
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
    State(state): State<AppState>,
) -> impl IntoResponse {
    match Connection::open(&state.db_path) {
        Ok(conn) => {
            // Query incident row
            let mut stmt = match conn.prepare(
                "SELECT incident_id, playbook_id, description, ts_created, severity, 
                        evidence_json, host, user, exe
                 FROM incidents 
                 WHERE incident_id = ?1"
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
                Ok((incident_id, playbook_id, description, ts, severity, evidence_json_opt, host, user, exe)) => {
                    // Parse evidence pointers
                    let mut evidence_ptrs: Vec<EvidencePtrJson> = vec![];
                    let mut timeline: Vec<TimelineEvent> = vec![];
                    
                    if let Some(json_str) = evidence_json_opt {
                        if let Ok(ptrs) = serde_json::from_str::<Vec<EvidencePtrJson>>(&json_str) {
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
                                                message: format!("Segment {} not found", ptr.segment_id),
                                                ptr: Some((ptr.segment_id.clone(), ptr.record_index)),
                                            }),
                                        });
                                    }
                                    Err(e) => {
                                        eprintln!("Warning: Failed to deref {}: {}", ptr.segment_id, e);
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
                                                message: format!("Failed to dereference evidence: {}", e),
                                                ptr: Some((ptr.segment_id.clone(), ptr.record_index)),
                                            }),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    
                    // Sort timeline by timestamp
                    timeline.sort_by_key(|e| e.ts);
                    
                    let narrative = description
                        .unwrap_or_else(|| format!("Incident {} fired from {}", incident_id, playbook_id));
                    
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
            let raw_event = serde_json::from_str(&event_json_str)
                .unwrap_or_else(|_| json!({}));
            
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
            let segment_path = telemetry_root
                .join("segments")
                .join(format!("{}.jsonl", ptr.segment_id));
            
            if !segment_path.exists() {
                // Task 6: Explicit missing segment error (never silent)
                eprintln!("[MISSING_EVIDENCE] Segment {} not found for ptr", ptr.segment_id);
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
                return Err(format!("{}:segment_missing", ErrorCode::SegmentMissing.as_str()));
            }
            
            if let Ok(contents) = fs::read_to_string(&segment_path) {
                for (idx, line) in contents.lines().enumerate() {
                    if idx == ptr.record_index as usize {
                        if let Ok(raw_event) = serde_json::from_str::<Value>(line) {
                            let exe = raw_event.get("exe")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let user = raw_event.get("user")
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
                 WHERE ts_ms = (SELECT MAX(ts_ms) FROM operator_events) LIMIT 1"
            ) {
                Ok(s) => s,
                Err(_) => return Json(SessionSummary {
                    session_id: "none".to_string(),
                    user: "unknown".to_string(),
                    host: "unknown".to_string(),
                    commands: vec![],
                    facts: vec![],
                    command_count: 0,
                }).into_response(),
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

            let _ = stmt_cmds.query_map(rusqlite::params![&session_id], |row| {
                if let Ok(json_str) = row.get::<_, String>(0) {
                    if let Ok(v) = serde_json::from_str::<Value>(&json_str) {
                        commands.push(SessionCommand {
                            cmd: v.get("cmd").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                            exit_code: v.get("exit_code").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
                            ts_ms: v.get("ts_ms").and_then(|x| x.as_u64()).unwrap_or(0),
                            duration_ms: v.get("duration_ms").and_then(|x| x.as_u64()).unwrap_or(0),
                        });
                    }
                }
                Ok(())
            }).ok();

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

            let _ = stmt_facts.query_map(rusqlite::params![&session_id], |row| {
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
                                            ts_ms: v.get("ts_ms").and_then(|x| x.as_u64()).unwrap_or(0),
                                            value: format!("{}/tcp", port),
                                            evidence_segment: row.get::<_, String>(1).unwrap_or_default(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }).ok();

            let command_count = commands.len() as u32;
            Json(SessionSummary {
                session_id,
                user,
                host,
                commands,
                facts,
                command_count,
            }).into_response()
        }
        Err(_) => Json(SessionSummary {
            session_id: "error".to_string(),
            user: "unknown".to_string(),
            host: "unknown".to_string(),
            commands: vec![],
            facts: vec![],
            command_count: 0,
        }).into_response(),
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
            
            let mut stmt = match conn.prepare(
                "SELECT event_json FROM operator_events WHERE session_id = ?"
            ) {
                Ok(s) => s,
                Err(_) => return Json(NextActionsResponse {
                    session_id: session_id.clone(),
                    actions: vec![],
                }).into_response(),
            };

            let _ = stmt.query_map(rusqlite::params![&session_id], |row| {
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
            }).ok();

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
                    reason: "SSH port detected (22/tcp open); attempt connection with verbosity".to_string(),
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
            }).into_response()
        }
        Err(_) => Json(NextActionsResponse {
            session_id: session_id.clone(),
            actions: vec![],
        }).into_response(),
    }
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
        .route("/api/sessions/latest", get(sessions_latest))
        .route("/api/sessions/:id/next_actions", get(sessions_next_actions))
        .with_state(state);
    
    let addr = "127.0.0.1:8000".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    // Setup graceful shutdown on signal
    let server = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal());
    
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
