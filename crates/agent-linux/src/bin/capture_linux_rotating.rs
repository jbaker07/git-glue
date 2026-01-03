//! capture_linux_rotating binary entry point
//! Linux eBPF + sensors_v2 capture with rotating segments

use std::path::PathBuf;
use std::time::Duration;

fn main() {
    eprintln!("capture_linux_rotating: Linux eBPF + sensors pipeline");
    
    // Get telemetry root from env or default
    let telemetry_root = std::env::var("EDR_TELEMETRY_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/lib/edr"));
    
    let segments_dir = telemetry_root.join("segments");
    
    // Create directories
    if let Err(e) = std::fs::create_dir_all(&segments_dir) {
        eprintln!("ERROR: Failed to create segments directory: {}", e);
        std::process::exit(1);
    }
    
    eprintln!("EDR_TELEMETRY_ROOT: {}", telemetry_root.display());
    eprintln!("Segments directory: {}", segments_dir.display());
    
    // Initialize capture
    let mut capture = match agent_linux::capture_linux_rotating::LinuxCapture::new(segments_dir.clone()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Failed to initialize capture: {}", e);
            std::process::exit(1);
        }
    };
    
    // Setup shutdown handler
    let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    ctrlc::set_handler(move || {
        eprintln!("\n[capture_linux_rotating] Shutdown signal received");
        shutdown_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    }).expect("Failed to set Ctrl-C handler");
    
    eprintln!("Beginning Linux capture loop (Ctrl-C to stop)...");
    
    // Main capture loop
    while !shutdown.load(std::sync::atomic::Ordering::SeqCst) {
        if let Err(e) = capture.poll_and_write() {
            eprintln!("[capture_linux_rotating] poll error: {}", e);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    
    eprintln!("[capture_linux_rotating] Capture stopped");
}
