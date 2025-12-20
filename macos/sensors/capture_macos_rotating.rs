/// capture_macos_rotating: Production OpenBSM → JSONL telemetry pipeline
/// 
/// Reads macOS audit trail from /dev/auditpipe, normalizes to versioned JSON,
/// writes atomically to rotating JSONL segments, maintains durable index.
/// 
/// PRODUCTION PATH ONLY: No synthetic events, no demo modes.

use std::fs::{self, OpenOptions, File};
use std::io::{Read, BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::process;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use serde::{Deserialize, Serialize};
use serde_json::json;
use forensic_hooks::audit_preflight;
use forensic_hooks::macos_detect::bsm_parser::BSMParser;
use uuid::Uuid;
use sha2::{Sha256, Digest};
fn get_telemetry_root() -> Result<PathBuf, String> {
    // Strict CLI precedence: --telemetry-root <path>
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() - 1 {
        if args[i] == "--telemetry-root" {
            return Ok(PathBuf::from(&args[i + 1]));
        }
    }
    
    // Environment: EDR_TELEMETRY_ROOT
    if let Ok(root) = std::env::var("EDR_TELEMETRY_ROOT") {
        return Ok(PathBuf::from(root));
    }
    
    // Default: cwd-relative
    std::env::current_dir()
        .map(|p| p.join("telemetry_output"))
        .map_err(|e| format!("Failed to determine cwd: {}", e))
}

fn get_max_segment_bytes() -> u64 {
    // EDR_MAX_SEGMENT_BYTES env var, default 256MB
    std::env::var("EDR_MAX_SEGMENT_BYTES")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(256 * 1024 * 1024)  // 256MB default
}

#[derive(Debug, Serialize, Deserialize)]
struct SegmentIndex {
    schema_version: u32,
    next_seq: u64,  // Monotonically increasing sequence number
    segments: Vec<SegmentMetadata>,
    last_updated_ts: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    prev_index_hash: Option<String>,  // Previous index SHA256
    #[serde(skip_serializing_if = "Option::is_none")]
    index_hash: Option<String>,  // Current index SHA256 (tamper-evident chain)
}

#[derive(Debug, Serialize, Deserialize)]
struct SegmentMetadata {
    seq: u64,  // Monotonic sequence number for this segment
    segment_id: String,
    rel_path: String,
    ts_first: u64,
    ts_last: u64,
    records: u32,
    size_bytes: u64,
    sha256_segment: String,  // SHA256 of segment file (required for integrity)
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,  // Deprecated alias for backward compat
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NormalizedEvent {
    schema_version: u32,
    ts_ms: u64,
    host: String,
    event_type: String,
    subject: SubjectInfo,
    process: ProcessInfo,
    payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_bsm: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SubjectInfo {
    uid: Option<u32>,
    gid: Option<u32>,
    auid: Option<u32>,
    euid: Option<u32>,
    egid: Option<u32>,
    session_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProcessInfo {
    pid: Option<u32>,
    ppid: Option<u32>,
    exe: Option<String>,
    argv: Option<Vec<String>>,
    cwd: Option<String>,
}

fn hostname() -> String {
    match std::process::Command::new("hostname")
        .output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string(),
        Err(_) => "unknown-host".to_string(),
    }
}

fn setup_signal_handler(shutdown_flag: Arc<AtomicBool>) {
    // Ctrl+C or SIGINT/SIGTERM triggers graceful shutdown
    let flag = shutdown_flag.clone();
    ctrlc::set_handler(move || {
        eprintln!("\n[capture] SIGINT/SIGTERM: graceful shutdown...");
        flag.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn load_or_create_index(index_path: &PathBuf) -> Result<SegmentIndex, String> {
    if let Ok(contents) = fs::read_to_string(index_path) {
        match serde_json::from_str::<SegmentIndex>(&contents) {
            Ok(idx) => return Ok(idx),
            Err(e) => eprintln!("Warning: Failed to parse index.json: {}", e),
        }
    }
    
    Ok(SegmentIndex {
        schema_version: 1,
        next_seq: 0,  // Will increment to 1 on first segment
        segments: Vec::new(),
        last_updated_ts: now_ms(),
        prev_index_hash: None,
        index_hash: None,
    })
}

fn validate_index(index: &SegmentIndex, segments_dir: &PathBuf) -> Result<(), String> {
    // Check seq is monotonically increasing
    let mut last_seq = 0u64;
    for seg in &index.segments {
        if seg.seq < last_seq {
            return Err(format!("Segment seq out of order: {} < {}", seg.seq, last_seq));
        }
        last_seq = seg.seq;
        
        // Check segment file exists
        let seg_path = segments_dir.join(&seg.rel_path);
        if !seg_path.exists() {
            eprintln!("WARNING: Index references missing segment: {}", seg.rel_path);
            // Don't fail, just warn (capture will continue with a degraded index)
        }
    }
    Ok(())
}

fn write_segment_atomic(
    segment_dir: &PathBuf,
    segment_id: &str,
    events: &[NormalizedEvent],
) -> Result<u64, String> {
    let segment_path = segment_dir.join(format!("{}.jsonl", segment_id));
    let temp_path = segment_dir.join(format!("{}.tmp", segment_id));
    
    // Write to temp
    let file = File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp segment: {}", e))?;
    let mut writer = BufWriter::new(file);
    
    let mut size_bytes = 0;
    for event in events {
        let json_line = serde_json::to_string(event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;
        writer.write_all(json_line.as_bytes())
            .map_err(|e| format!("Failed to write event: {}", e))?;
        writer.write_all(b"\n")
            .map_err(|e| format!("Failed to write newline: {}", e))?;
        size_bytes += json_line.len() as u64 + 1;
    }
    
    drop(writer);
    
    // Sync to disk
    std::process::Command::new("sync")
        .output()
        .ok();
    
    // Atomic rename
    fs::rename(&temp_path, &segment_path)
        .map_err(|e| format!("Failed to rename segment: {}", e))?;
    
    Ok(size_bytes)
}

fn compute_sha256_file(path: &PathBuf) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open file for SHA256: {}", e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buffer[..n]),
            Err(e) => return Err(format!("Failed to read for SHA256: {}", e)),
        }
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

fn compute_sha256_string(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn update_index_atomic(
    index_path: &PathBuf,
    mut index: SegmentIndex,
    segment_id: &str,
    rel_path: String,
    ts_first: u64,
    ts_last: u64,
    records: u32,
    size_bytes: u64,
    sha256_segment: String,
) -> Result<(), String> {
    let temp_path = index_path.with_extension("json.tmp");
    
    // Assign monotonic seq to new segment
    let seq = index.next_seq;
    index.next_seq += 1;
    
    // Add/update segment metadata
    if let Some(existing) = index.segments.iter_mut().find(|s| s.segment_id == segment_id) {
        existing.ts_last = ts_last;
        existing.records = records;
        existing.size_bytes = size_bytes;
        existing.sha256_segment = sha256_segment.clone();
    } else {
        index.segments.push(SegmentMetadata {
            seq,
            segment_id: segment_id.to_string(),
            rel_path,
            ts_first,
            ts_last,
            records,
            size_bytes,
            sha256_segment: sha256_segment.clone(),
            sha256: Some(sha256_segment.clone()),  // Backward compat
        });
    }
    
    index.last_updated_ts = now_ms();
    
    // Store previous hash
    index.prev_index_hash = index.index_hash.clone();
    
    // Write temp
    let json_str = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("Failed to serialize index: {}", e))?;
    
    // Compute current index hash (for tamper-evident chain)
    index.index_hash = Some(compute_sha256_string(&json_str));
    let final_json_str = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("Failed to serialize index with hash: {}", e))?;
    
    fs::write(&temp_path, &final_json_str)
        .map_err(|e| format!("Failed to write temp index: {}", e))?;
    
    // Sync
    std::process::Command::new("sync")
        .output()
        .ok();
    
    // Atomic rename
    fs::rename(&temp_path, index_path)
        .map_err(|e| format!("Failed to rename index: {}", e))?;
    
    // P0: Write index.json.bak after successful update
    let bak_path = index_path.with_extension("json.bak");
    let _ = fs::write(&bak_path, final_json_str);
    
    Ok(())
}

fn validate_root_permissions() -> Result<(), String> {
    if unsafe { libc::geteuid() } != 0 {
        return Err("capture_macos_rotating requires root. Run with sudo.".to_string());
    }
    Ok(())
}

fn cleanup_stale_temp_files(segments_dir: &PathBuf) {
    // Remove any orphan .tmp segment or index files from crashed writes
    match fs::read_dir(segments_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        if name_str.ends_with(".tmp") {
                            if let Err(e) = fs::remove_file(&path) {
                                eprintln!("[capture] Warning: Failed to remove stale {}: {}", name_str, e);
                            } else {
                                eprintln!("[capture] Cleaned up stale: {}", name_str);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("[capture] Warning: Failed to scan segments dir: {}", e);
        }
    }
}

fn get_build_id() -> String {
    // Use EDR_BUILD_ID env var if set, else fallback to CARGO_PKG_VERSION
    std::env::var("EDR_BUILD_ID")
        .unwrap_or_else(|_| {
            env!("CARGO_PKG_VERSION").to_string()
        })
}

fn write_heartbeat(telemetry_root: &PathBuf, seq: u64) -> Result<(), String> {
    let heartbeat = json!({
        "ts_ms": now_ms(),
        "pid": process::id(),
        "seq": seq,
        "schema_version": 1,
        "capture_build_id": get_build_id()
    });
    
    let heartbeat_path = telemetry_root.join("capture_heartbeat.json");
    let temp_path = telemetry_root.join("capture_heartbeat.json.tmp");
    
    // Write to temp file
    let json_str = serde_json::to_string(&heartbeat)
        .map_err(|e| format!("Heartbeat JSON error: {}", e))?;
    
    fs::write(&temp_path, json_str)
        .map_err(|e| format!("Heartbeat write error: {}", e))?;
    
    // Fsync the file
    let f = OpenOptions::new()
        .write(true)
        .open(&temp_path)
        .map_err(|e| format!("Heartbeat fsync open error: {}", e))?;
    
    f.sync_all()
        .map_err(|e| format!("Heartbeat fsync error: {}", e))?;
    
    drop(f);
    
    // Atomic rename
    fs::rename(&temp_path, &heartbeat_path)
        .map_err(|e| format!("Heartbeat rename error: {}", e))?;
    
    Ok(())
}

fn main() {
    eprintln!("capture_macos_rotating: Production OpenBSM pipeline");
    
    // Check for --once flag (exit after first segment for testing)
    let once_mode = std::env::args().any(|arg| arg == "--once");
    
    // Check audit preflight - MANDATORY
    let audit_status = audit_preflight::check_audit_preflight();
    if !audit_status.enabled {
        eprintln!("FATAL: Audit not available");
        if let Some(e) = &audit_status.error_msg {
            eprintln!("{}", e);
        }
        process::exit(1);
    }
    eprintln!("[audit] Preflight check passed");
    
    // Validate root
    if let Err(e) = validate_root_permissions() {
        eprintln!("ERROR: {}", e);
        process::exit(1);
    }
    
    // Resolve telemetry root
    let telemetry_root = match get_telemetry_root() {
        Ok(root) => {
            eprintln!("TELEMETRY_ROOT: {}", root.display());
            root
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
    };
    
    // Create directories
    let segments_dir = telemetry_root.join("segments");
    if let Err(e) = fs::create_dir_all(&segments_dir) {
        eprintln!("ERROR: Failed to create segments directory: {}", e);
        process::exit(1);
    }
    
    // Clean up any stale .tmp files from previous crashes
    cleanup_stale_temp_files(&segments_dir);
    
    let index_path = telemetry_root.join("index.json");
    
    // Load and validate existing index
    if let Ok(index) = load_or_create_index(&index_path) {
        if let Err(e) = validate_index(&index, &segments_dir) {
            eprintln!("ERROR: Index validation failed: {}", e);
            process::exit(1);
        }
    }
    
    eprintln!("Opening /dev/auditpipe for reading...");
    
    // PRODUCTION PATH: Read from /dev/auditpipe
    let mut auditpipe = match OpenOptions::new()
        .read(true)
        .open("/dev/auditpipe") {
        Ok(f) => {
            eprintln!("Connected to /dev/auditpipe");
            f
        }
        Err(e) => {
            eprintln!("ERROR: Failed to open /dev/auditpipe: {}", e);
            eprintln!("Note: audit must be enabled. Run: praudit -l /dev/auditpipe");
            process::exit(1);
        }
    };
    
    let host = hostname();
    let mut buffer = vec![0u8; 65536];
    let mut current_segment_events: Vec<NormalizedEvent> = Vec::new();
    let mut current_segment_bytes: u64 = 0;
    let mut last_segment_ts = now_ms();
    let segment_rotation_ms = 60_000; // 1 minute rotation (P0-2)
    let max_segment_bytes = get_max_segment_bytes();
    let mut bsm_parser = BSMParser::new();
    let mut last_heartbeat_ts = now_ms();
    let heartbeat_interval_ms = 5_000; // 5 second heartbeat
    let mut current_seq = 0u64;
    
    eprintln!("[segment] Max segment size: {} bytes ({:.2} MB)", 
              max_segment_bytes, 
              max_segment_bytes as f64 / (1024.0 * 1024.0));
    
    // Setup graceful shutdown flag
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    setup_signal_handler(shutdown_flag.clone());
    
    eprintln!("Beginning OpenBSM ingestion with BSM parsing...");
    eprintln!("Events will be normalized and written to: {}", segments_dir.display());
    eprintln!("(Press Ctrl+C to gracefully shutdown)");
    
    loop {
        // Check for graceful shutdown signal
        if shutdown_flag.load(Ordering::Relaxed) {
            eprintln!("[capture] Flushing final segment before exit...");
            if !current_segment_events.is_empty() {
                let segment_id = format!("seg_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
                match write_segment_atomic(&segments_dir, &segment_id, &current_segment_events) {
                    Ok(size) => {
                        let ts_first = current_segment_events.first().map(|e| e.ts_ms).unwrap_or(0);
                        let ts_last = current_segment_events.last().map(|e| e.ts_ms).unwrap_or(0);
                        let record_count = current_segment_events.len() as u32;
                        
                        let segment_path = segments_dir.join(format!("{}.jsonl", segment_id));
                        let sha256 = compute_sha256_file(&segment_path).unwrap_or_else(|_| "unknown".to_string());
                        
                        if let Ok(mut index) = load_or_create_index(&index_path) {
                            let rel_path = format!("segments/{}.jsonl", segment_id);
                            if let Err(e) = update_index_atomic(
                                &index_path,
                                index,
                                &segment_id,
                                rel_path,
                                ts_first,
                                ts_last,
                                record_count,
                                size,
                                sha256,
                            ) {
                                eprintln!("[capture] ERROR updating final index: {}", e);
                            }
                        }
                        eprintln!("[capture] Flushed {} events", record_count);
                    }
                    Err(e) => {
                        eprintln!("[capture] ERROR writing final segment: {}", e);
                    }
                }
            }
            eprintln!("[capture] Clean exit");
            process::exit(0);
        }
        match auditpipe.read(&mut buffer) {
            Ok(0) => {
                // EOF (shouldn't happen with auditpipe, but handle gracefully)
                eprintln!("EOF from auditpipe");
                break;
            }
            Ok(n) => {
                // Feed raw bytes to BSM parser
                bsm_parser.feed(&buffer[..n]);
                
                // Try to extract complete records from the buffer
                let mut record_count = 0;
                while let Some((parsed_record, _bytes_consumed)) = bsm_parser.try_parse_record() {
                    record_count += 1;
                    
                    // Convert parsed BSM to normalized JSON event
                    let normalized = parsed_record.to_normalized_json(&host);
                    
                    let event = NormalizedEvent {
                        schema_version: 1,
                        ts_ms: normalized["ts_ms"].as_u64().unwrap_or_else(|| now_ms()),
                        host: host.clone(),
                        event_type: normalized["event_type"].as_str().unwrap_or("unknown").to_string(),
                        subject: SubjectInfo {
                            uid: normalized["subject"]["uid"].as_u64().map(|v| v as u32),
                            gid: normalized["subject"]["gid"].as_u64().map(|v| v as u32),
                            auid: normalized["subject"]["auid"].as_u64().map(|v| v as u32),
                            euid: normalized["subject"]["euid"].as_u64().map(|v| v as u32),
                            egid: normalized["subject"]["egid"].as_u64().map(|v| v as u32),
                            session_id: normalized["subject"]["session_id"].as_u64().map(|v| v as u32),
                        },
                        process: ProcessInfo {
                            pid: normalized["process"]["pid"].as_u64().map(|v| v as u32),
                            ppid: normalized["process"]["ppid"].as_u64().map(|v| v as u32),
                            exe: normalized["process"]["exe"].as_str().map(|s| s.to_string()),
                            argv: normalized["process"]["argv"].as_array().map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            }),
                            cwd: normalized["process"]["cwd"].as_str().map(|s| s.to_string()),
                        },
                        payload: normalized.get("payload").cloned().unwrap_or(json!({})),
                        raw_bsm: None,
                    };
                    
                    current_segment_events.push(event.clone());
                    // Estimate size: rough JSON serialization
                    if let Ok(json_str) = serde_json::to_string(&event) {
                        current_segment_bytes += json_str.len() as u64 + 1;  // +1 for newline
                    }
                }
                
                if record_count > 0 {
                    eprintln!("[auditpipe] Parsed {} BSM records from {} bytes", record_count, n);
                }
            }
            Err(e) => {
                eprintln!("ERROR reading auditpipe: {}", e);
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        
        // Rotate segment if necessary (time OR size threshold)
        let now = now_ms();
        let should_rotate = (now - last_segment_ts > segment_rotation_ms) || 
                           (current_segment_bytes > max_segment_bytes);
        
        if should_rotate && !current_segment_events.is_empty() {
            let rotate_reason = if current_segment_bytes > max_segment_bytes {
                format!("size threshold ({:.2} MB / {:.2} MB)", 
                        current_segment_bytes as f64 / (1024.0 * 1024.0),
                        max_segment_bytes as f64 / (1024.0 * 1024.0))
            } else {
                "time threshold".to_string()
            };
            
            // Use UUID for collision-resistant segment IDs (no reuse on restart)
            let segment_id = format!("seg_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
            eprintln!("Rotating segment ({}): {}", rotate_reason, segment_id);
            
            match write_segment_atomic(&segments_dir, &segment_id, &current_segment_events) {
                Ok(size) => {
                    let ts_first = current_segment_events.first().map(|e| e.ts_ms).unwrap_or(0);
                    let ts_last = current_segment_events.last().map(|e| e.ts_ms).unwrap_or(0);
                    let record_count = current_segment_events.len() as u32;
                    
                    let segment_path = segments_dir.join(format!("{}.jsonl", segment_id));
                    let sha256 = compute_sha256_file(&segment_path).unwrap_or_else(|_| "unknown".to_string());
                    
                    if let Ok(mut index) = load_or_create_index(&index_path) {
                        let rel_path = format!("segments/{}.jsonl", segment_id);
                        current_seq = index.next_seq;  // Capture seq before increment
                        if let Err(e) = update_index_atomic(
                            &index_path,
                            index,
                            &segment_id,
                            rel_path,
                            ts_first,
                            ts_last,
                            record_count,
                            size,
                            sha256,
                        ) {
                            eprintln!("ERROR updating index: {}", e);
                        }
                    }
                    
                    current_segment_events.clear();
                    current_segment_bytes = 0;  // Reset size counter
                    last_segment_ts = now;
                    eprintln!("  Segment: {} events, {} bytes ({:.2} MB)", 
                              record_count, size, size as f64 / (1024.0 * 1024.0));
                    
                    // P0-1: Exit after first segment in --once mode (for testing)
                    if once_mode {
                        eprintln!("[capture] --once mode: exiting after first segment");
                        process::exit(0);
                    }
                }
                Err(e) => {
                    eprintln!("ERROR writing segment: {}", e);
                }
            }
        }
        
        // Write heartbeat every 5 seconds
        let now = now_ms();
        if now - last_heartbeat_ts > heartbeat_interval_ms {
            let _ = write_heartbeat(&telemetry_root, current_seq);
            last_heartbeat_ts = now;
        }
    }
}
