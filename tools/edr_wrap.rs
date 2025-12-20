/// edr_wrap: Minimal operator wrapper for workflow telemetry capture
/// 
/// Usage: edr_wrap [--session <id>] -- <command> [args...]
/// 
/// Captures: ts_ms, host, user, cwd, cmd, argv, stdout, stderr, exit_code, duration_ms, session_id, sensor_kind
/// Writes: $TELEMETRY_ROOT/operator/segments/seg_{uuid}.jsonl + $TELEMETRY_ROOT/operator/index.json
/// 
/// Reuses atomic segment + index patterns from capture_macos_rotating (no unsafe duplication).
/// Shares same stream/segment infrastructure for multi-stream ingestion in edr_locald.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use serde_json::json;
use uuid::Uuid;
use sha2::{Sha256, Digest};

fn get_telemetry_root() -> PathBuf {
    if let Ok(root) = std::env::var("EDR_TELEMETRY_ROOT") {
        return PathBuf::from(root);
    }
    std::env::current_dir()
        .map(|p| p.join("telemetry_output"))
        .unwrap_or_else(|_| PathBuf::from("./telemetry_output"))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn get_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn get_host() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string())
}

fn get_cwd() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "/".to_string())
}

fn get_session_id() -> String {
    let user = get_user();
    let host = get_host();
    let ts_ms = now_ms();
    let bucket = (ts_ms / 1800000) as u64;  // 30-min window for stable session grouping
    format!("{}@{}:{}", user, host, bucket)
}

#[derive(serde::Serialize)]
struct OperatorEvent {
    schema_version: u32,
    ts_ms: u64,
    host: String,
    user: String,
    cwd: String,
    cmd: String,
    argv: Vec<String>,
    stdout: String,
    stderr: String,
    exit_code: i32,
    duration_ms: u64,
    session_id: String,
    sensor_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_hint: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SegmentIndex {
    schema_version: u32,
    next_seq: u64,
    segments: Vec<SegmentRef>,
    last_updated_ts: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SegmentRef {
    segment_id: String,
    seq: u64,
    path: String,
    sha256_segment: String,
    ts_created: u64,
}

fn compute_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn write_segment_atomic(
    segment_dir: &PathBuf,
    segment_id: &str,
    content: &str,
) -> Result<(), String> {
    let segment_path = segment_dir.join(format!("{}.jsonl", segment_id));
    let temp_path = segment_dir.join(format!("{}.tmp", segment_id));

    // Write to temp file
    fs::write(&temp_path, content)
        .map_err(|e| format!("Failed to write temp segment: {}", e))?;

    // Fsync for durability
    let f = std::fs::OpenOptions::new()
        .write(true)
        .open(&temp_path)
        .map_err(|e| format!("Failed to fsync open: {}", e))?;
    f.sync_all()
        .map_err(|e| format!("Failed to fsync: {}", e))?;
    drop(f);

    // Atomic rename
    fs::rename(&temp_path, &segment_path)
        .map_err(|e| format!("Failed to rename segment: {}", e))?;

    Ok(())
}

fn update_index_atomic(
    telemetry_root: &PathBuf,
    new_segment: SegmentRef,
) -> Result<(), String> {
    let index_path = telemetry_root.join("operator").join("index.json");
    let temp_path = index_path.with_extension("json.tmp");

    // Create parent directory if needed
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create index dir: {}", e))?;
    }

    let mut index = if index_path.exists() {
        let contents = fs::read_to_string(&index_path)
            .map_err(|e| format!("Failed to read operator index.json: {}", e))?;
        serde_json::from_str::<SegmentIndex>(&contents)
            .unwrap_or(SegmentIndex {
                schema_version: 1,
                next_seq: 1,
                segments: Vec::new(),
                last_updated_ts: now_ms(),
            })
    } else {
        SegmentIndex {
            schema_version: 1,
            next_seq: 1,
            segments: Vec::new(),
            last_updated_ts: now_ms(),
        }
    };

    // Append new segment with next seq
    let mut seg = new_segment;
    seg.seq = index.next_seq;
    index.segments.push(seg);
    index.next_seq += 1;
    index.last_updated_ts = now_ms();

    // Write to temp
    let json_str = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("Failed to serialize index: {}", e))?;
    fs::write(&temp_path, json_str)
        .map_err(|e| format!("Failed to write temp index: {}", e))?;

    // Fsync
    let f = std::fs::OpenOptions::new()
        .write(true)
        .open(&temp_path)
        .map_err(|e| format!("Failed to fsync index: {}", e))?;
    f.sync_all()
        .map_err(|e| format!("Failed to fsync index: {}", e))?;
    drop(f);

    // Atomic rename
    fs::rename(&temp_path, &index_path)
        .map_err(|e| format!("Failed to rename index: {}", e))?;

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse: edr_wrap [--session <id>] -- <cmd> [args...]
    let (session_id, cmd_args) = if let Some(sep_idx) = args.iter().position(|a| a == "--") {
        let before_sep = &args[1..sep_idx];
        let after_sep = &args[sep_idx + 1..];

        let session_id = if before_sep.len() >= 2 && before_sep[0] == "--session" {
            before_sep[1].clone()
        } else {
            get_session_id()
        };

        if after_sep.is_empty() {
            eprintln!("Error: No command specified after --");
            std::process::exit(1);
        }

        (session_id, after_sep.to_vec())
    } else {
        eprintln!("Usage: edr_wrap [--session <id>] -- <command> [args...]");
        std::process::exit(1);
    };

    // Prepare environment
    let telemetry_root = get_telemetry_root();
    let segment_dir = telemetry_root.join("operator").join("segments");
    fs::create_dir_all(&segment_dir).ok();

    // Execute command with timing
    let start = Instant::now();
    let output = Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let duration_ms = start.elapsed().as_millis() as u64;

    let (exit_code, stdout_str, stderr_str) = match output {
        Ok(out) => (
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        ),
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            std::process::exit(1);
        }
    };

    // Build event with schema_version and sensor_kind
    let event = OperatorEvent {
        schema_version: 1,
        ts_ms: now_ms(),
        host: get_host(),
        user: get_user(),
        cwd: get_cwd(),
        cmd: cmd_args[0].clone(),
        argv: cmd_args.clone(),
        stdout: stdout_str,
        stderr: stderr_str,
        exit_code,
        duration_ms,
        session_id: session_id.clone(),
        sensor_kind: "operator".to_string(),
        tool_hint: None,
    };

    // Write as JSONL
    let event_json = serde_json::to_string(&event).unwrap_or_default();
    let segment_id = format!("seg_{}", Uuid::new_v4());
    let sha256_event = compute_sha256(&event_json);

    // Write segment
    if let Err(e) = write_segment_atomic(&segment_dir, &segment_id, &format!("{}\n", event_json)) {
        eprintln!("Failed to write segment: {}", e);
        std::process::exit(1);
    }

    // Update index
    let seg_ref = SegmentRef {
        segment_id: segment_id.clone(),
        seq: 0,  // Will be filled by update_index_atomic
        path: format!("operator/segments/{}.jsonl", segment_id),
        sha256_segment: sha256_event,
        ts_created: now_ms(),
    };

    if let Err(e) = update_index_atomic(&telemetry_root, seg_ref) {
        eprintln!("Failed to update index: {}", e);
        std::process::exit(1);
    }

    eprintln!("[edr_wrap] Recorded: {} → {}.jsonl (session: {})", cmd_args[0], segment_id, session_id);
}
