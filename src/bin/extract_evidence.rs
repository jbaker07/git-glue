//! extract_evidence - Extract raw event from DB/segments by EvidencePtr
//! Accepts EvidencePtr string, returns raw event JSON

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" {
        eprintln!("extract_evidence - Extract raw event by EvidencePtr");
        eprintln!("Usage: extract_evidence <evidence_ptr> [--db-path PATH] [--segments-dir DIR]");
        eprintln!("  <evidence_ptr>        EvidencePtr string (e.g., 'seg:abc123:offset:456')");
        eprintln!("  --db-path PATH        Path to incident database");
        eprintln!("  --segments-dir DIR    Directory containing segment files");
        return Ok(());
    }

    let evidence_ptr = &args[1];
    let mut db_path = PathBuf::from("/var/lib/edr/incidents.db");
    let mut segments_dir = PathBuf::from("/var/lib/edr/segments");

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--db-path" => {
                if i + 1 < args.len() {
                    db_path = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--segments-dir" => {
                if i + 1 < args.len() {
                    segments_dir = PathBuf::from(&args[i + 1]);
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

    extract_evidence_impl(evidence_ptr, &db_path, &segments_dir)?;
    Ok(())
}

fn extract_evidence_impl(
    evidence_ptr: &str,
    db_path: &PathBuf,
    segments_dir: &PathBuf,
) -> Result<()> {
    eprintln!("[extract_evidence] Looking up: {}", evidence_ptr);
    eprintln!("[extract_evidence] DB: {:?}", db_path);
    eprintln!("[extract_evidence] Segments: {:?}", segments_dir);

    // Parse evidence_ptr format: "seg:<segment_id>:offset:<offset>" or similar
    if evidence_ptr.starts_with("seg:") {
        // Format: seg:segment_id:offset:offset_value
        eprintln!("[extract_evidence] Parsed segment reference");
        // TODO: implement segment lookup and raw event extraction
        eprintln!(r#"{{"error": "Not yet implemented"}}"#);
    } else {
        eprintln!("[extract_evidence] Invalid EvidencePtr format");
        eprintln!(r#"{{"error": "Invalid format"}}"#);
        std::process::exit(1);
    }

    Ok(())
}
