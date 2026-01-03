//! edr_locald - Local runtime engine for correlation and playbook execution
//! Ingests core::Event, runs correlation/playbooks, upserts incidents/state

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--help" {
        eprintln!("edr_locald - Local runtime engine");
        eprintln!("Usage: edr_locald [--playbooks DIR] [--state-db PATH]");
        eprintln!("  --playbooks DIR       Path to playbook definitions");
        eprintln!("  --state-db PATH       Path to incident state database");
        return Ok(());
    }

    let mut playbooks_dir = PathBuf::from("./playbooks");
    let mut state_db_path = PathBuf::from("/var/lib/edr/state.db");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--playbooks" => {
                if i + 1 < args.len() {
                    playbooks_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--state-db" => {
                if i + 1 < args.len() {
                    state_db_path = PathBuf::from(&args[i + 1]);
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

    eprintln!("[edr_locald] playbooks={:?}, state_db={:?}", playbooks_dir, state_db_path);
    eprintln!("[edr_locald] Starting local runtime engine");

    Ok(())
}
