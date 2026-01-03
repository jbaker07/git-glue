/// Fixture generator for smoke, attack_chain, and flood scenarios
use crate::core::{Event, EvidencePtr};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Generate minimal JSONL fixture (smoke test)
pub fn gen_smoke_fixture<P: AsRef<Path>>(path: P) -> Result<usize> {
    let mut file = File::create(path)?;

    // 3 minimal events
    let events = vec![
        Event {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            host: "localhost".to_string(),
            tags: vec!["process".to_string(), "exec".to_string()],
            proc_key: Some("bash_1234".to_string()),
            file_key: None,
            identity_key: None,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "mock_0".to_string(),
                segment_id: 0,
                record_index: 0,
            }),
            fields: {
                let mut m = BTreeMap::new();
                m.insert("process_id".to_string(), serde_json::json!(1234));
                m.insert("process_name".to_string(), serde_json::json!("bash"));
                m.insert("user".to_string(), serde_json::json!("root"));
                m
            },
        },
        Event {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            host: "localhost".to_string(),
            tags: vec!["file".to_string(), "open".to_string()],
            proc_key: Some("bash_1234".to_string()),
            file_key: Some("file_/etc/passwd".to_string()),
            identity_key: None,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "mock_0".to_string(),
                segment_id: 0,
                record_index: 1,
            }),
            fields: {
                let mut m = BTreeMap::new();
                m.insert("process_id".to_string(), serde_json::json!(1234));
                m.insert("path".to_string(), serde_json::json!("/etc/passwd"));
                m.insert("user".to_string(), serde_json::json!("root"));
                m
            },
        },
        Event {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            host: "localhost".to_string(),
            tags: vec!["network".to_string(), "connect".to_string()],
            proc_key: Some("curl_1234".to_string()),
            file_key: None,
            identity_key: None,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "mock_0".to_string(),
                segment_id: 0,
                record_index: 2,
            }),
            fields: {
                let mut m = BTreeMap::new();
                m.insert("process_id".to_string(), serde_json::json!(1234));
                m.insert("process_name".to_string(), serde_json::json!("curl"));
                m.insert("user".to_string(), serde_json::json!("root"));
                m.insert("remote_ip".to_string(), serde_json::json!("8.8.8.8"));
                m
            },
        },
    ];

    let count = events.len();
    for evt in events {
        let json = serde_json::to_string(&evt)?;
        writeln!(file, "{}", json)?;
    }

    Ok(count)
}

/// Generate attack chain fixture (sequential suspicious events)
pub fn gen_attack_chain_fixture<P: AsRef<Path>>(path: P) -> Result<usize> {
    let mut file = File::create(path)?;

    let sequence = vec![
        ("ssh.login", 456),
        ("privilege.escalation", 457),
        ("process.inject", 458),
        ("file.exfiltrate", 459),
        ("network.connect", 460),
    ];

    let mut count = 0;
    for (tags_str, pid) in sequence {
        let tags: Vec<String> = tags_str.split('.').map(|s| s.to_string()).collect();
        let evt = Event {
            ts_ms: chrono::Utc::now().timestamp_millis() + (count as i64 * 1000),
            host: "localhost".to_string(),
            tags,
            proc_key: Some(format!("proc_{}", pid)),
            file_key: None,
            identity_key: None,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "mock_0".to_string(),
                segment_id: 0,
                record_index: count as u32,
            }),
            fields: {
                let mut m = BTreeMap::new();
                m.insert("process_id".to_string(), serde_json::json!(pid));
                m
            },
        };

        let json = serde_json::to_string(&evt)?;
        writeln!(file, "{}", json)?;
        count += 1;
    }

    Ok(count)
}

/// Generate flood fixture (many events for throughput testing)
pub fn gen_flood_fixture<P: AsRef<Path>>(path: P, count: usize) -> Result<usize> {
    let mut file = File::create(path)?;

    let event_tags = vec![
        vec!["file".to_string(), "open".to_string()],
        vec!["file".to_string(), "write".to_string()],
        vec!["network".to_string(), "sendto".to_string()],
        vec!["process".to_string(), "fork".to_string()],
    ];

    for i in 0..count {
        let evt = Event {
            ts_ms: chrono::Utc::now().timestamp_millis() + (i as i64 * 100),
            host: "localhost".to_string(),
            tags: event_tags[i % event_tags.len()].clone(),
            proc_key: Some(format!("proc_{}", 2000 + i)),
            file_key: None,
            identity_key: None,
            evidence_ptr: Some(EvidencePtr {
                stream_id: "mock_0".to_string(),
                segment_id: 0,
                record_index: i as u32,
            }),
            fields: {
                let mut m = BTreeMap::new();
                m.insert("process_id".to_string(), serde_json::json!(2000 + i as u32));
                m.insert(
                    "process_name".to_string(),
                    serde_json::json!(format!("proc_{}", i)),
                );
                m
            },
        };

        let json = serde_json::to_string(&evt)?;
        writeln!(file, "{}", json)?;
    }

    Ok(count)
}

// Tests can be added once tempfile is available as test dependency
