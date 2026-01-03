//! edr_server - HTTP API and UI server
//! Reads DB/state, serves incidents/timelines/evidence queries

use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--help" {
        eprintln!("edr_server - HTTP API and UI server");
        eprintln!("Usage: edr_server [--listen ADDR:PORT] [--state-db PATH]");
        eprintln!("  --listen ADDR:PORT    Listen address (default: 127.0.0.1:8080)");
        eprintln!("  --state-db PATH       Path to state database");
        return Ok(());
    }

    let mut listen = "127.0.0.1:8080".to_string();
    let mut state_db = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--listen" => {
                if i + 1 < args.len() {
                    listen = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--state-db" => {
                if i + 1 < args.len() {
                    state_db = Some(args[i + 1].clone());
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

    eprintln!("[edr_server] listen={}, state_db={:?}", listen, state_db);
    eprintln!("[edr_server] Starting HTTP API server");

    Ok(())
}
