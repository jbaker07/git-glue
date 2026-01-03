// src/bin/ui_server.rs
// E2E verification server wrapper - Workbench + EDR API server

use axum::{routing::{get, post, put}, Router};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};

use forensic_hooks::server::workbench_handlers::{self, WorkbenchState};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        eprintln!("ui_server - Attack Documentation Workbench + EDR API server");
        eprintln!("Usage: ui_server [OPTIONS]");
        eprintln!("  --telemetry-root PATH  Telemetry root directory (default: $TELEMETRY_ROOT or /tmp/telemetry)");
        eprintln!("  --port PORT            Listen port (default: 8000)");
        eprintln!("  --help                 Show this message");
        return;
    }

    let mut telemetry_root =
        std::env::var("TELEMETRY_ROOT").unwrap_or_else(|_| "/tmp/telemetry".to_string());
    let mut port = 8000u16;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--telemetry-root" | "-t" => {
                if i + 1 < args.len() {
                    telemetry_root = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(8000);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    eprintln!("[ui_server] Attack Documentation Workbench");
    eprintln!("[ui_server] TELEMETRY_ROOT={}", telemetry_root);
    eprintln!("[ui_server] Port={}", port);

    // Build EDR state
    let edr_state = forensic_hooks::server::ui_server::AppState {
        telemetry_root: PathBuf::from(&telemetry_root),
        db_path: PathBuf::from(format!("{}/analysis.db", telemetry_root)),
    };

    // Build Workbench state
    let workbench_state = WorkbenchState::new(PathBuf::from(&telemetry_root));

    // CORS layer for browser access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build app with all routes
    let app = Router::new()
        // EDR endpoints
        .route("/api/health", get(forensic_hooks::server::ui_server::health))
        
        // Workbench Session endpoints
        .route("/api/workbench/sessions", get(workbench_handlers::sessions_list))
        .route("/api/workbench/session/start", post(workbench_handlers::session_start))
        .route("/api/workbench/session/:id", get(workbench_handlers::session_get))
        .route("/api/workbench/session/:id/stop", post(workbench_handlers::session_stop))
        .route("/api/workbench/session/:id/pause", post(workbench_handlers::session_pause))
        .route("/api/workbench/session/:id/resume", post(workbench_handlers::session_resume))
        .route("/api/workbench/session/:id/marker", post(workbench_handlers::session_add_marker))
        
        // Workbench Document endpoints
        .route("/api/workbench/documents", get(workbench_handlers::documents_list))
        .route("/api/workbench/document", post(workbench_handlers::document_create))
        .route("/api/workbench/document/:id", get(workbench_handlers::document_get))
        .route("/api/workbench/document/:id/section", put(workbench_handlers::document_update_section))
        .route("/api/workbench/document/:id/technique", put(workbench_handlers::document_update_technique))
        .route("/api/workbench/document/:id/timeline", post(workbench_handlers::document_add_timeline_entry))
        .route("/api/workbench/document/:id/timeline/:entry_id/star", put(workbench_handlers::document_star_entry))
        .route("/api/workbench/document/:id/timeline/:entry_id/annotate", put(workbench_handlers::document_annotate_entry))
        .route("/api/workbench/document/:id/export", post(workbench_handlers::document_export))
        
        // MITRE ATT&CK reference endpoints
        .route("/api/workbench/mitre", get(workbench_handlers::mitre_list))
        .route("/api/workbench/mitre/search", get(workbench_handlers::mitre_search))
        .route("/api/workbench/mitre/:id", get(workbench_handlers::mitre_get))
        
        .layer(cors)
        .with_state(workbench_state)
        // EDR state as a separate layer (health endpoint)
        .route("/api/health", get(forensic_hooks::server::ui_server::health))
        .with_state(edr_state);

    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .unwrap_or_else(|_| "127.0.0.1:8000".parse().unwrap());

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ui_server] Failed to bind {}: {}", addr, e);
            return;
        }
    };

    eprintln!("[ui_server] Listening on http://{}", addr);
    eprintln!("[ui_server] Endpoints:");
    eprintln!("  GET  /api/health                    - System health check");
    eprintln!("  POST /api/workbench/session/start   - Start capture session");
    eprintln!("  POST /api/workbench/document        - Create new document");
    eprintln!("  GET  /api/workbench/mitre           - List MITRE techniques");
    eprintln!("[ui_server] (Ctrl-C to shutdown)");

    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        eprintln!("[ui_server] Error: {}", e);
    }

    eprintln!("[ui_server] Clean shutdown");
}

async fn shutdown_signal() {
    use tokio::signal;
    let _ = signal::ctrl_c().await;
}

