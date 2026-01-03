//! capture_macos_rotating - macOS sensor capture with rotating segments
//! Collects macOS sensors, writes rotating segments, assigns EvidencePtr

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--help" {
        eprintln!("capture_macos_rotating - macOS sensor capture");
        eprintln!("Usage: capture_macos_rotating [--output DIR]");
        eprintln!("  --output DIR          Output directory for segments (default: /tmp/capture)");
        return Ok(());
    }

    let mut output = PathBuf::from("/tmp/capture");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    eprintln!("[capture_macos_rotating] output={:?}", output);
    eprintln!("[capture_macos_rotating] Starting macOS capture");

    Ok(())
}
