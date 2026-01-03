//! EDR Desktop Application - Tauri wrapper for local EDR workbench
//! 
//! This app:
//! - Launches the ui_server backend as a child process
//! - Waits for /api/health to become ready
//! - Opens the workbench UI in the Tauri window
//! - Provides clean shutdown (SIGTERM/SIGKILL child on exit)
//! - Runs fully offline with no cloud calls

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;
mod logging;
mod port;

use backend::BackendManager;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::RwLock;

/// Application state shared across commands
pub struct AppState {
    backend: Arc<RwLock<BackendManager>>,
}

/// Get backend status for UI display
#[tauri::command]
async fn get_status(state: State<'_, AppState>) -> Result<backend::StatusInfo, String> {
    let backend = state.backend.read().await;
    Ok(backend.status().await)
}

/// Get the base URL for API calls
#[tauri::command]
async fn get_api_base_url(state: State<'_, AppState>) -> Result<String, String> {
    let backend = state.backend.read().await;
    Ok(backend.api_base_url())
}

/// Open the telemetry folder in system file browser
#[tauri::command]
async fn open_telemetry_folder(state: State<'_, AppState>) -> Result<(), String> {
    let backend = state.backend.read().await;
    let path = backend.telemetry_root();
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

/// Manually restart the backend (for troubleshooting)
#[tauri::command]
async fn restart_backend(state: State<'_, AppState>) -> Result<backend::StatusInfo, String> {
    let mut backend = state.backend.write().await;
    backend.restart().await?;
    Ok(backend.status().await)
}

fn main() {
    // Initialize logging
    let _guard = logging::init_logging();
    
    tracing::info!("EDR Desktop starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Spawn async backend initialization
            tauri::async_runtime::spawn(async move {
                match BackendManager::new().await {
                    Ok(mut manager) => {
                        // Start the backend
                        if let Err(e) = manager.start().await {
                            tracing::error!("Failed to start backend: {}", e);
                            show_error_dialog(&app_handle, &format!(
                                "Failed to start backend server:\n\n{}\n\n\
                                Please check:\n\
                                1. The ui_server binary exists in target/release/\n\
                                2. Port {} is not in use\n\
                                3. You have write access to the telemetry folder",
                                e, manager.port()
                            ));
                            return;
                        }

                        // Wait for health check
                        if let Err(e) = manager.wait_for_ready(std::time::Duration::from_secs(10)).await {
                            tracing::error!("Backend health check failed: {}", e);
                            show_error_dialog(&app_handle, &format!(
                                "Backend server started but health check failed:\n\n{}\n\n\
                                The server may still be initializing. Try restarting the app.",
                                e
                            ));
                            // Continue anyway - the server might still work
                        }

                        tracing::info!("Backend ready at {}", manager.api_base_url());
                        
                        // Store in app state
                        let state = AppState {
                            backend: Arc::new(RwLock::new(manager)),
                        };
                        app_handle.manage(state);

                        // Navigate main window to workbench
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let backend = app_handle.state::<AppState>();
                            let backend_guard = backend.backend.blocking_read();
                            let url = format!("{}/ui/index.html", backend_guard.api_base_url());
                            drop(backend_guard);
                            
                            if let Err(e) = window.eval(&format!(
                                "window.location.href = '{}';",
                                url
                            )) {
                                tracing::warn!("Failed to navigate window: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize backend manager: {}", e);
                        show_error_dialog(&app_handle, &format!(
                            "Failed to initialize EDR Desktop:\n\n{}\n\n\
                            Please ensure the application has proper permissions.",
                            e
                        ));
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                tracing::info!("Window close requested, shutting down backend...");
                
                // Shutdown backend on window close
                if let Some(state) = window.try_state::<AppState>() {
                    let backend = state.backend.clone();
                    tauri::async_runtime::block_on(async {
                        let mut manager = backend.write().await;
                        if let Err(e) = manager.shutdown().await {
                            tracing::error!("Error during backend shutdown: {}", e);
                        }
                    });
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_api_base_url,
            open_telemetry_folder,
            restart_backend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_error_dialog(app: &tauri::AppHandle, message: &str) {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .message(message)
        .title("EDR Desktop Error")
        .blocking_show();
}
