/// Portable telemetry root resolution
/// Rule: CLI arg → env var → relative ./telemetry_output
/// NEVER walk exe path or hardcode EDR-platform

use std::path::PathBuf;

pub struct TelemetryPaths {
    pub root: PathBuf,
    pub segments_dir: PathBuf,
    pub index_file: PathBuf,
    pub db_path: PathBuf,
    pub playbooks_dir: PathBuf,
}

impl TelemetryPaths {
    pub fn new(
        cli_root: Option<&str>,
        cli_playbooks: Option<&str>,
        cli_segments: Option<&str>,
        cli_index: Option<&str>,
        cli_db: Option<&str>,
    ) -> Self {
        // Resolve telemetry root
        let root = if let Some(r) = cli_root {
            PathBuf::from(r)
        } else if let Ok(r) = std::env::var("EDR_TELEMETRY_ROOT") {
            PathBuf::from(r)
        } else {
            // Default: relative to current working directory
            std::env::current_dir()
                .map(|p| p.join("telemetry_output"))
                .unwrap_or_else(|_| PathBuf::from("./telemetry_output"))
        };

        // Resolve sub-paths
        let segments_dir = if let Some(s) = cli_segments {
            PathBuf::from(s)
        } else {
            root.join("segments")
        };

        let index_file = if let Some(i) = cli_index {
            PathBuf::from(i)
        } else {
            root.join("index.json")
        };

        let db_path = if let Some(d) = cli_db {
            PathBuf::from(d)
        } else {
            root.join("analysis.db")
        };

        let playbooks_dir = if let Some(p) = cli_playbooks {
            PathBuf::from(p)
        } else {
            // Try ./playbooks/macos first (platform-scoped), then under telemetry_root/playbooks/macos
            let local = PathBuf::from("./playbooks/macos");
            if local.exists() {
                local
            } else {
                root.join("playbooks").join("macos")
            }
        };

        TelemetryPaths {
            root,
            segments_dir,
            index_file,
            db_path,
            playbooks_dir,
        }
    }

    pub fn display_config(&self) {
        eprintln!("Telemetry Configuration:");
        eprintln!("  root: {}", self.root.display());
        eprintln!("  segments_dir: {}", self.segments_dir.display());
        eprintln!("  index_file: {}", self.index_file.display());
        eprintln!("  db_path: {}", self.db_path.display());
        eprintln!("  playbooks_dir: {}", self.playbooks_dir.display());
    }
}
