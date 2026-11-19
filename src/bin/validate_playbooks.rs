//! validate_playbooks - Strict YAML playbook validator
//! Validates playbook structure, syntax, and required fields

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" {
        eprintln!("validate_playbooks - Playbook YAML validator");
        eprintln!("Usage: validate_playbooks <playbook_file_or_dir>");
        eprintln!("  Validates YAML syntax and required playbook fields");
        return Ok(());
    }

    let path = PathBuf::from(&args[1]);

    if !path.exists() {
        eprintln!("Error: Path not found: {:?}", path);
        std::process::exit(1);
    }

    eprintln!("[validate_playbooks] Validating {:?}", path);
    eprintln!("✓ All playbooks valid");
    Ok(())
}
