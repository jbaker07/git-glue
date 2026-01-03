//! Tier-0 procfs process sensor
//! Reads /proc filesystem to emit process snapshot events
//! Pure event emitter with no side effects

use anyhow::Result;
use crate::core::Event;
use std::collections::BTreeMap;
use std::fs;
use serde_json::json;

/// Poll processes from /proc and return events
pub fn poll_processes(host: &str) -> Result<Vec<Event>> {
    let mut events = Vec::new();
    
    // Read /proc directory
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(filename) = entry.file_name().into_string() {
                // Look for numeric directories (PIDs)
                if let Ok(pid) = filename.parse::<u32>() {
                    if let Ok(event) = read_process_event(host, pid) {
                        events.push(event);
                    }
                }
            }
        }
    }
    
    Ok(events)
}

/// Read a single process from /proc/{pid} and convert to Event
fn read_process_event(host: &str, pid: u32) -> Result<Event> {
    let procfs_path = format!("/proc/{}", pid);
    
    // Read /proc/{pid}/stat for basic process info
    let stat_path = format!("{}/stat", procfs_path);
    let stat = fs::read_to_string(&stat_path)?;
    
    // Parse stat line: pid (comm) state ppid ...
    let parts: Vec<&str> = stat.split_whitespace().collect();
    if parts.len() < 4 {
        anyhow::bail!("Invalid stat format for pid {}", pid);
    }
    
    let ppid = parts[3].parse::<u32>().unwrap_or(0);
    let state = parts[2].chars().next().unwrap_or('?');
    
    // Read /proc/{pid}/status for uid/gid
    let status_path = format!("{}/status", procfs_path);
    let status = fs::read_to_string(&status_path)?;
    
    let mut uid = 0u32;
    let mut gid = 0u32;
    for line in status.lines() {
        if line.starts_with("Uid:") {
            if let Some(first) = line.split_whitespace().nth(1) {
                uid = first.parse().unwrap_or(0);
            }
        }
        if line.starts_with("Gid:") {
            if let Some(first) = line.split_whitespace().nth(1) {
                gid = first.parse().unwrap_or(0);
            }
        }
    }
    
    // Read /proc/{pid}/cmdline for process command
    let cmdline_path = format!("{}/cmdline", procfs_path);
    let cmdline_bytes = fs::read(&cmdline_path).unwrap_or_default();
    let args: Vec<String> = cmdline_bytes
        .split(|&b| b == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).to_string())
        .collect();
    
    let exe = args.first().cloned().unwrap_or_default();
    
    // Build Event
    let mut fields = BTreeMap::new();
    fields.insert("pid".to_string(), json!(pid));
    fields.insert("ppid".to_string(), json!(ppid));
    fields.insert("uid".to_string(), json!(uid));
    fields.insert("gid".to_string(), json!(gid));
    fields.insert("state".to_string(), json!(state.to_string()));
    fields.insert("exe".to_string(), json!(exe));
    fields.insert("args".to_string(), json!(args));
    
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    
    Ok(Event {
        ts_ms: now_ms,
        host: host.to_string(),
        tags: vec!["linux".to_string(), "process".to_string(), "procfs".to_string()],
        proc_key: None,  // Will be populated by capture layer if needed
        file_key: None,
        identity_key: None,
        evidence_ptr: crate::EvidencePtr {
            stream_id: String::new(),  // Will be populated by capture layer
            segment_id: 0,
            record_index: 0,
        },
        fields,
    })
}
