// server/workbench_handlers.rs
// HTTP handlers for the Attack Documentation Workbench

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::workbench::{
    api::*,
    document::*,
    export::*,
    session::*,
};

// ============================================================================
// Workbench State
// ============================================================================

#[derive(Clone)]
pub struct WorkbenchState {
    pub telemetry_root: PathBuf,
    pub sessions: Arc<RwLock<HashMap<String, CaptureSession>>>,
    pub documents: Arc<RwLock<HashMap<String, Document>>>,
}

impl WorkbenchState {
    pub fn new(telemetry_root: PathBuf) -> Self {
        Self {
            telemetry_root,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// ============================================================================
// Session Endpoints
// ============================================================================

/// POST /api/workbench/session/start
/// Start a new capture session
pub async fn session_start(
    State(state): State<WorkbenchState>,
) -> impl IntoResponse {
    let session = CaptureSession::new();
    let session_id = session.id.clone();
    
    // Start the session
    let mut started_session = session;
    started_session.start();
    
    if let Ok(mut sessions) = state.sessions.write() {
        sessions.insert(session_id.clone(), started_session);
    }
    
    Json(ApiResponse::ok(json!({
        "session_id": session_id,
        "status": "capturing",
        "started_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    })))
}

/// POST /api/workbench/session/:id/stop
/// Stop a capture session
pub async fn session_stop(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
) -> impl IntoResponse {
    if let Ok(mut sessions) = state.sessions.write() {
        if let Some(session) = sessions.get_mut(&id) {
            session.stop();
            return Json(ApiResponse::ok(json!({
                "session_id": id,
                "status": "stopped",
                "markers": session.markers.len(),
                "terminal_commands": session.terminal_commands.len()
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Session not found")))
        .into_response()
}

/// POST /api/workbench/session/:id/pause
pub async fn session_pause(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
) -> impl IntoResponse {
    if let Ok(mut sessions) = state.sessions.write() {
        if let Some(session) = sessions.get_mut(&id) {
            session.pause();
            return Json(ApiResponse::ok(json!({
                "session_id": id,
                "status": "paused"
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Session not found")))
        .into_response()
}

/// POST /api/workbench/session/:id/resume
pub async fn session_resume(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
) -> impl IntoResponse {
    if let Ok(mut sessions) = state.sessions.write() {
        if let Some(session) = sessions.get_mut(&id) {
            session.resume();
            return Json(ApiResponse::ok(json!({
                "session_id": id,
                "status": "capturing"
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Session not found")))
        .into_response()
}

/// POST /api/workbench/session/:id/marker
/// Add a marker to the session (Important, PhaseStart, etc.)
#[derive(Debug, Deserialize)]
pub struct AddMarkerRequest {
    pub marker_type: String,
    pub note: Option<String>,
}

pub async fn session_add_marker(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
    Json(req): Json<AddMarkerRequest>,
) -> impl IntoResponse {
    let marker_type = match req.marker_type.as_str() {
        "important" => MarkerType::Important,
        "phase_start" => MarkerType::PhaseStart,
        "phase_end" => MarkerType::PhaseEnd,
        "note" => MarkerType::Note,
        "attack_start" => MarkerType::AttackStart,
        "attack_end" => MarkerType::AttackEnd,
        "terminal_focus" => MarkerType::TerminalFocus,
        "browser_focus" => MarkerType::BrowserFocus,
        _ => MarkerType::Note,
    };
    
    if let Ok(mut sessions) = state.sessions.write() {
        if let Some(session) = sessions.get_mut(&id) {
            session.add_marker(marker_type, req.note);
            return Json(ApiResponse::ok(json!({
                "session_id": id,
                "markers_count": session.markers.len()
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Session not found")))
        .into_response()
}

/// GET /api/workbench/session/:id
/// Get session status and summary
pub async fn session_get(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
) -> impl IntoResponse {
    if let Ok(sessions) = state.sessions.read() {
        if let Some(session) = sessions.get(&id) {
            return Json(ApiResponse::ok(json!({
                "session_id": session.id,
                "state": format!("{:?}", session.state),
                "started_at": session.started_at,
                "stopped_at": session.stopped_at,
                "markers": session.markers.iter().map(|m| json!({
                    "timestamp": m.timestamp,
                    "marker_type": format!("{:?}", m.marker_type),
                    "note": m.note
                })).collect::<Vec<_>>(),
                "terminal_commands_count": session.terminal_commands.len(),
                "browser_activities_count": session.browser_activities.len()
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Session not found")))
        .into_response()
}

// ============================================================================
// Document Endpoints
// ============================================================================

/// POST /api/workbench/document
/// Create a new document
pub async fn document_create(
    State(state): State<WorkbenchState>,
    Json(req): Json<CreateDocumentRequest>,
) -> impl IntoResponse {
    let mut doc = Document::new(&req.title);
    
    if let Some(author) = req.author {
        doc.author = author;
    }
    if let Some(tags) = req.tags {
        doc.tags = tags;
    }
    
    let doc_id = doc.id.clone();
    
    if let Ok(mut documents) = state.documents.write() {
        documents.insert(doc_id.clone(), doc);
    }
    
    Json(ApiResponse::ok(json!({
        "document_id": doc_id,
        "created": true
    })))
}

/// GET /api/workbench/document/:id
/// Get document by ID
pub async fn document_get(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
) -> impl IntoResponse {
    if let Ok(documents) = state.documents.read() {
        if let Some(doc) = documents.get(&id) {
            return Json(ApiResponse::ok(json!(doc)));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Document not found")))
        .into_response()
}

/// PUT /api/workbench/document/:id/section
/// Update a document section
pub async fn document_update_section(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
    Json(req): Json<UpdateSectionRequest>,
) -> impl IntoResponse {
    if let Ok(mut documents) = state.documents.write() {
        if let Some(doc) = documents.get_mut(&id) {
            match req.section.as_str() {
                "summary" => doc.summary.set_user(&req.content),
                "impact" => doc.impact.set_user(&req.content),
                "recommendations" => doc.recommendations.set_user(&req.content),
                "title" => doc.title = req.content,
                _ => return (StatusCode::BAD_REQUEST, 
                    Json(ApiResponse::<Value>::err("Invalid section name"))).into_response(),
            }
            return Json(ApiResponse::ok(json!({
                "document_id": id,
                "section": req.section,
                "updated": true
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Document not found")))
        .into_response()
}

/// POST /api/workbench/document/:id/timeline
/// Add a timeline entry
pub async fn document_add_timeline_entry(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
    Json(req): Json<AddTimelineEntryRequest>,
) -> impl IntoResponse {
    if let Ok(mut documents) = state.documents.write() {
        if let Some(doc) = documents.get_mut(&id) {
            let entry = TimelineEntry {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: req.timestamp,
                title: req.title,
                description: req.description.unwrap_or_default(),
                event_type: req.event_type,
                evidence_ptr: req.evidence_ptr,
                included: true,
                starred: false,
                annotation: None,
            };
            doc.timeline.entries.push(entry);
            doc.timeline.sort_by_time();
            
            return Json(ApiResponse::ok(json!({
                "document_id": id,
                "timeline_entries": doc.timeline.entries.len()
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Document not found")))
        .into_response()
}

/// PUT /api/workbench/document/:id/timeline/:entry_id/star
/// Star/unstar a timeline entry
pub async fn document_star_entry(
    Path((doc_id, entry_id)): Path<(String, String)>,
    State(state): State<WorkbenchState>,
    Json(req): Json<StarEventRequest>,
) -> impl IntoResponse {
    if let Ok(mut documents) = state.documents.write() {
        if let Some(doc) = documents.get_mut(&doc_id) {
            if let Some(entry) = doc.timeline.entries.iter_mut().find(|e| e.id == entry_id) {
                entry.starred = req.starred;
                return Json(ApiResponse::ok(json!({
                    "entry_id": entry_id,
                    "starred": entry.starred
                })));
            }
            return (StatusCode::NOT_FOUND, 
                Json(ApiResponse::<Value>::err("Timeline entry not found"))).into_response();
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Document not found")))
        .into_response()
}

/// PUT /api/workbench/document/:id/timeline/:entry_id/annotate
/// Add annotation to a timeline entry
pub async fn document_annotate_entry(
    Path((doc_id, entry_id)): Path<(String, String)>,
    State(state): State<WorkbenchState>,
    Json(req): Json<AnnotateEventRequest>,
) -> impl IntoResponse {
    if let Ok(mut documents) = state.documents.write() {
        if let Some(doc) = documents.get_mut(&doc_id) {
            if let Some(entry) = doc.timeline.entries.iter_mut().find(|e| e.id == entry_id) {
                entry.annotation = Some(req.annotation.clone());
                return Json(ApiResponse::ok(json!({
                    "entry_id": entry_id,
                    "annotation": req.annotation
                })));
            }
            return (StatusCode::NOT_FOUND, 
                Json(ApiResponse::<Value>::err("Timeline entry not found"))).into_response();
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Document not found")))
        .into_response()
}

/// PUT /api/workbench/document/:id/technique
/// Update MITRE technique mapping
pub async fn document_update_technique(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
    Json(req): Json<UpdateTechniqueRequest>,
) -> impl IntoResponse {
    if let Ok(mut documents) = state.documents.write() {
        if let Some(doc) = documents.get_mut(&id) {
            if let Some(tech_id) = req.technique_id {
                doc.technique.technique_id = tech_id;
            }
            if let Some(tech_name) = req.technique_name {
                doc.technique.technique_name = tech_name;
            }
            if let Some(tactic) = req.tactic {
                doc.technique.tactic = tactic;
            }
            if let Some(confidence) = req.confidence {
                doc.technique.confidence = confidence;
            }
            if let Some(notes) = req.notes {
                doc.technique.notes = notes;
            }
            return Json(ApiResponse::ok(json!({
                "document_id": id,
                "technique_id": doc.technique.technique_id,
                "technique_name": doc.technique.technique_name
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Document not found")))
        .into_response()
}

// ============================================================================
// Export Endpoints
// ============================================================================

/// POST /api/workbench/document/:id/export
/// Export document to HTML/Markdown/JSON
pub async fn document_export(
    Path(id): Path<String>,
    State(state): State<WorkbenchState>,
    Json(req): Json<ExportRequest>,
) -> impl IntoResponse {
    if let Ok(documents) = state.documents.read() {
        if let Some(doc) = documents.get(&id) {
            let options = req.options.unwrap_or_default();
            let exporter = Exporter::new(options);
            
            let (content, content_type) = match req.format {
                ExportFormat::Html => (exporter.to_html(doc), "text/html"),
                ExportFormat::Markdown => (exporter.to_markdown(doc), "text/markdown"),
                ExportFormat::Json => (serde_json::to_string_pretty(doc).unwrap_or_default(), "application/json"),
                ExportFormat::Pdf => {
                    // PDF requires HTML -> PDF conversion (placeholder)
                    let html = exporter.to_html(doc);
                    return Json(ApiResponse::ok(json!({
                        "format": "pdf",
                        "note": "PDF export requires browser print. Use HTML and print to PDF.",
                        "html": html
                    })));
                }
            };
            
            return Json(ApiResponse::ok(json!({
                "format": format!("{:?}", req.format),
                "content": content,
                "content_type": content_type
            })));
        }
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Document not found")))
        .into_response()
}

// ============================================================================
// MITRE ATT&CK Reference Endpoints
// ============================================================================

/// GET /api/workbench/mitre/search?q=query
#[derive(Debug, Deserialize)]
pub struct MitreSearchQuery {
    pub q: String,
}

pub async fn mitre_search(
    axum::extract::Query(query): axum::extract::Query<MitreSearchQuery>,
) -> impl IntoResponse {
    let results = search_techniques(&query.q);
    Json(ApiResponse::ok(json!({
        "query": query.q,
        "results": results.iter().map(|t| json!({
            "id": t.id,
            "name": t.name,
            "tactic": t.tactic,
            "description": t.description
        })).collect::<Vec<_>>()
    })))
}

/// GET /api/workbench/mitre/:id
/// Get a specific MITRE technique by ID
pub async fn mitre_get(
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Some(tech) = get_technique(&id) {
        return Json(ApiResponse::ok(json!({
            "id": tech.id,
            "name": tech.name,
            "tactic": tech.tactic,
            "description": tech.description
        })));
    }
    
    (StatusCode::NOT_FOUND, Json(ApiResponse::<Value>::err("Technique not found")))
        .into_response()
}

/// GET /api/workbench/mitre
/// List all MITRE techniques
pub async fn mitre_list() -> impl IntoResponse {
    let techniques: Vec<_> = MITRE_TECHNIQUES.iter().map(|t| json!({
        "id": t.id,
        "name": t.name,
        "tactic": t.tactic
    })).collect();
    
    Json(ApiResponse::ok(json!({
        "count": techniques.len(),
        "techniques": techniques
    })))
}

// ============================================================================
// List Endpoints
// ============================================================================

/// GET /api/workbench/sessions
/// List all capture sessions
pub async fn sessions_list(
    State(state): State<WorkbenchState>,
) -> impl IntoResponse {
    if let Ok(sessions) = state.sessions.read() {
        let list: Vec<_> = sessions.values().map(|s| json!({
            "id": s.id,
            "state": format!("{:?}", s.state),
            "started_at": s.started_at,
            "stopped_at": s.stopped_at,
            "markers_count": s.markers.len()
        })).collect();
        
        return Json(ApiResponse::ok(json!({
            "count": list.len(),
            "sessions": list
        })));
    }
    
    Json(ApiResponse::ok(json!({
        "count": 0,
        "sessions": []
    })))
}

/// GET /api/workbench/documents
/// List all documents
pub async fn documents_list(
    State(state): State<WorkbenchState>,
) -> impl IntoResponse {
    if let Ok(documents) = state.documents.read() {
        let list: Vec<_> = documents.values().map(|d| json!({
            "id": d.id,
            "title": d.title,
            "author": d.author,
            "created_at": d.created_at,
            "tags": d.tags,
            "timeline_entries": d.timeline.entries.len()
        })).collect();
        
        return Json(ApiResponse::ok(json!({
            "count": list.len(),
            "documents": list
        })));
    }
    
    Json(ApiResponse::ok(json!({
        "count": 0,
        "documents": []
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workbench_state_new() {
        let state = WorkbenchState::new(PathBuf::from("/tmp/test"));
        assert!(state.sessions.read().unwrap().is_empty());
        assert!(state.documents.read().unwrap().is_empty());
    }
}
