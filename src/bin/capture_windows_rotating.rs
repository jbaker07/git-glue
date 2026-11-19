// src/bin/capture_windows_rotating.rs
// Windows event log capture binary entry point
// Supports: --self-test-windows, --output DIR, --mock-events PATH

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut output = PathBuf::from(".");
    let mut mock_events = None;
    let mut self_test = false;

    // Parse args
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--mock-events" => {
                if i + 1 < args.len() {
                    mock_events = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--self-test-windows" => {
                self_test = true;
                i += 1;
            }
            "--help" | "-h" => {
                eprintln!("capture_windows_rotating - Windows event log capture");
                eprintln!("Usage: capture_windows_rotating [OPTIONS]");
                eprintln!("Options:");
                eprintln!("  --output DIR              Output directory for segments (default: .)");
                eprintln!("  --mock-events PATH        Load mock JSONL events from file");
                eprintln!("  --self-test-windows       Run 30s self-test (fail if errors or (Windows && no events))");
                eprintln!("  --help                    Show this help");
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                i += 1;
            }
        }
    }

    eprintln!(
        "[capture_windows_rotating] output={:?}, mock_events={:?}, self_test={}",
        output, mock_events, self_test
    );

    #[cfg(target_os = "windows")]
    {
        if self_test {
            return run_self_test();
        }

        eprintln!("[capture_windows_rotating] Note: WEVTAPI reader not yet implemented");
        eprintln!(
            "[capture_windows_rotating] Run with --self-test-windows to validate collection system"
        );

        // TODO: Implement real Windows capture when WEVTAPI is available
        // For now, just create output dir
        std::fs::create_dir_all(&output)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        if self_test {
            return run_self_test();
        }

        eprintln!("[capture_windows_rotating] This binary is Windows-only");
        eprintln!("[capture_windows_rotating] On non-Windows platforms, use capture_linux_rotating instead");
        eprintln!(
            "[capture_windows_rotating] (Self-test mode works on any platform with env vars set)"
        );
    }

    Ok(())
}

fn run_self_test() -> anyhow::Result<()> {
    let host = forensic_hooks::linux::host::HostCtx::new();
    forensic_hooks::windows::self_test::run_windows_selftest(&host)?;
    Ok(())
}
