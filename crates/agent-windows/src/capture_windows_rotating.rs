// windows/sensors/capture_windows_rotating.rs
// Windows event log capture binary (JSONL segments + index, heartbeat)
// Analogous to linux/capture_linux_rotating.rs
// Polls Windows Event Logs (Sysmon, Security, System, etc.) with bounded work

#![cfg(target_os = "windows")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureHeartbeat {
    pub ts: DateTime<Utc>,
    pub transport: String,
    pub source: String,
    pub events_read_total: u64,
    pub events_read_delta: u64,
    pub parse_failed_total: u64,
    pub parse_failed_delta: u64,
    pub last_event_ts: Option<DateTime<Utc>>,
    pub status: String,
    // Canonical event counters (9 types + 2 breakdown)
    pub credential_access_count: u64,
    pub discovery_count: u64,
    pub exfiltration_count: u64,
    pub network_connection_count: u64,
    pub persistence_change_count: u64,
    pub defense_evasion_count: u64,
    pub process_injection_count: u64,
    pub auth_event_count: u64,
    pub script_exec_count: u64,
    pub archive_tool_exec_count: u64,
    pub staging_write_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentIndex {
    pub segment_id: String,
    pub start_ts: DateTime<Utc>,
    pub end_ts: DateTime<Utc>,
    pub record_count: u64,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeartbeatMetrics {
    pub source: String,
    pub events_read_total: u64,
    pub events_read_delta: u64,
    pub parse_failed_total: u64,
    pub parse_failed_delta: u64,
    pub last_event_ts: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct WindowsCaptureConfig {
    pub root_dir: PathBuf,
    pub segment_rotation_records: u64,
    pub heartbeat_interval_secs: u64,
}

pub struct WindowsEventCapture {
    config: WindowsCaptureConfig,
    segment_count: u64,
    event_count: u64,
    collector: crate::sensors::EvtxCollector,
    metrics: BTreeMap<String, HeartbeatMetrics>,
    // Attack surface event counters
    proc_exec_count: u64,
    priv_escalation_count: u64,
    proc_access_count: u64,
    asr_block_count: u64,
    wmi_persistence_count: u64,
    persistence_service_count: u64,
    persistence_task_count: u64,
    log_clear_count: u64,
    remote_logon_rdp_count: u64,
    remote_winrm_count: u64,
    decode_failed_count: u64,
    // Primitive counters - Original 4 types
    cred_access_count: u64,
    discovery_exec_count: u64,
    archive_tool_exec_count: u64,
    staging_write_count: u64,
    net_connect_prim_count: u64,
    // Extended 3 canonical primitives (Windows parity)
    persistence_change_count: u64,
    defense_evasion_count: u64,
    script_exec_count: u64,
}

impl WindowsEventCapture {
    pub fn new(root_dir: PathBuf) -> Self {
        fs::create_dir_all(&root_dir).ok();

        Self {
            config: WindowsCaptureConfig {
                root_dir,
                segment_rotation_records: 10000,
                heartbeat_interval_secs: 5,
            },
            segment_count: 0,
            event_count: 0,
            collector: crate::sensors::EvtxCollector::new(),
            metrics: BTreeMap::new(),
            proc_exec_count: 0,
            priv_escalation_count: 0,
            proc_access_count: 0,
            asr_block_count: 0,
            wmi_persistence_count: 0,
            persistence_service_count: 0,
            persistence_task_count: 0,
            log_clear_count: 0,
            remote_logon_rdp_count: 0,
            remote_winrm_count: 0,
            decode_failed_count: 0,
            cred_access_count: 0,
            discovery_exec_count: 0,
            archive_tool_exec_count: 0,
            staging_write_count: 0,
            net_connect_prim_count: 0,
            persistence_change_count: 0,
            defense_evasion_count: 0,
            script_exec_count: 0,
        }
    }

    /// Create capture with mock JSONL events
    pub fn with_mock_events(root_dir: PathBuf, fixture_path: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(&root_dir)?;

        eprintln!(
            "[capture_windows] Mock mode: reading JSONL events from {:?}",
            fixture_path
        );

        // Load events from JSONL fixture
        let (events, skipped) = forensic_hooks::mock::load_events_jsonl(fixture_path)?;
        eprintln!(
            "[capture_windows] Mock loaded {} events (skipped {})",
            events.len(),
            skipped
        );

        // Write segment with mock events
        let segment_id = "windows_000000";
        let segment_path = root_dir.join(format!("{}.jsonl", segment_id));

        let mut segment_content = String::new();
        for evt in events {
            if let Ok(json_str) = serde_json::to_string(&evt) {
                segment_content.push_str(&json_str);
                segment_content.push('\n');
            }
        }

        fs::write(&segment_path, segment_content)?;

        // Write heartbeat
        let heartbeat = serde_json::json!({
            "ts": Utc::now(),
            "transport": "mock",
            "events_read_total": events.len(),
            "parse_failed_total": skipped,
            "status": "complete"
        });
        fs::write(
            root_dir.join("heartbeat.json"),
            serde_json::to_string(&heartbeat)?,
        )?;

        eprintln!("[capture_windows] Mock capture complete");

        Ok(Self {
            config: WindowsCaptureConfig {
                root_dir,
                segment_rotation_records: 10000,
                heartbeat_interval_secs: 5,
            },
            segment_count: 1,
            event_count: events.len() as u64,
            collector: crate::sensors::EvtxCollector::new(),
            metrics: BTreeMap::new(),
            proc_exec_count: 0,
            priv_escalation_count: 0,
            proc_access_count: 0,
            asr_block_count: 0,
            wmi_persistence_count: 0,
            persistence_service_count: 0,
            persistence_task_count: 0,
            log_clear_count: 0,
            remote_logon_rdp_count: 0,
            remote_winrm_count: 0,
            decode_failed_count: 0,
            cred_access_count: 0,
            discovery_exec_count: 0,
            archive_tool_exec_count: 0,
            staging_write_count: 0,
            net_connect_prim_count: 0,
            persistence_change_count: 0,
            defense_evasion_count: 0,
            script_exec_count: 0,
        })
    }

    /// Poll all enabled event log sources and write segments
    /// CRITICAL: Derives primitives → dedup → EvidencePtr assignment → write
    pub fn poll_and_write(&mut self) -> anyhow::Result<()> {
        let host = crate::host::HostCtx::new();
        let segment_id = format!("evtx_{:06}", self.segment_count);
        let segment_path = self.config.root_dir.join(format!("{}.jsonl", segment_id));

        // Call unified collector that handles all sources + gating
        // Returns events with None evidence_ptr
        let mut all_events = crate::sensors::collect::collect_all(&host);

        // STEP 1: Derive canonical primitives from base events (process, network, file)
        let mut derived_events = Vec::new();
        for event in &all_events {
            let primitives = crate::sensors::primitives::derive_primitive_events(event);
            derived_events.extend(primitives);
        }

        // Merge derived primitives into events
        all_events.extend(derived_events);

        // STEP 2: Sort deterministically (by timestamp + type + pid)
        sort_events(&mut all_events);

        // STEP 3: Dedup derived primitives (same primitive from same source shouldn't repeat)
        let mut seen_derived = std::collections::HashSet::new();
        let mut deduped_events = Vec::new();

        for event in all_events {
            // Check if this is a derived primitive by tags (all 9 canonical types)
            let is_derived = event.tags.contains(&"credential_access".to_string())
                || event.tags.contains(&"discovery".to_string())
                || event.tags.contains(&"exfiltration".to_string())
                || event.tags.contains(&"network_connection".to_string())
                || event.tags.contains(&"persistence_change".to_string())
                || event.tags.contains(&"defense_evasion".to_string())
                || event.tags.contains(&"process_injection".to_string())
                || event.tags.contains(&"auth_event".to_string())
                || event.tags.contains(&"script_exec".to_string());

            if is_derived {
                // Generate dedup key for derived primitives
                let dedup_key = generate_dedup_key(&event);

                if seen_derived.contains(&dedup_key) {
                    // Skip duplicate derived primitive from this poll window
                    continue;
                }
                seen_derived.insert(dedup_key);
            }

            deduped_events.push(event);
        }

        let mut all_events = deduped_events;

        // Count attack surface events before EvidencePtr assignment
        for event in &all_events {
            match event
                .fields
                .get(crate::core::event_keys::EVENT_KIND)
                .and_then(|v| v.as_str())
            {
                Some("proc_exec") => self.proc_exec_count += 1,
                Some("priv_escalation") => self.priv_escalation_count += 1,
                Some("proc_access") => self.proc_access_count += 1,
                Some("asr_block") => self.asr_block_count += 1,
                Some("wmi_persistence") => self.wmi_persistence_count += 1,
                Some("persistence_service") => self.persistence_service_count += 1,
                Some("persistence_task") => self.persistence_task_count += 1,
                Some("log_clear") => self.log_clear_count += 1,
                Some("remote_logon_rdp") => self.remote_logon_rdp_count += 1,
                Some("remote_winrm") => self.remote_winrm_count += 1,
                _ => {}
            }

            // Count primitive types
            if event.tags.contains(&"credential_access".to_string()) {
                self.cred_access_count += 1;
            } else if event.tags.contains(&"discovery".to_string()) {
                self.discovery_exec_count += 1;
            } else if event.tags.contains(&"exfiltration".to_string()) {
                if event
                    .fields
                    .contains_key(crate::core::event_keys::ARCHIVE_TOOL)
                {
                    self.archive_tool_exec_count += 1;
                } else if event
                    .fields
                    .contains_key(crate::core::event_keys::FILE_PATH)
                {
                    self.staging_write_count += 1;
                }
            } else if event.tags.contains(&"network_connection".to_string()) {
                self.net_connect_prim_count += 1;
            } else if event.tags.contains(&"persistence_change".to_string()) {
                self.persistence_change_count += 1;
            } else if event.tags.contains(&"defense_evasion".to_string()) {
                self.defense_evasion_count += 1;
            } else if event.tags.contains(&"script_exec".to_string()) {
                self.script_exec_count += 1;
            }
        }

        // STEP 4: ASSIGN EvidencePtr HERE ONLY - uses segment state
        let mut all_events = self.assign_evidence_ptrs_with_state(all_events, self.segment_count);

        // STEP 5: Validate canonical primitives contract
        for event in &all_events {
            if event.tags.contains(&"credential_access".to_string())
                || event.tags.contains(&"discovery".to_string())
                || event.tags.contains(&"exfiltration".to_string())
                || event.tags.contains(&"network_connection".to_string())
                || event.tags.contains(&"persistence_change".to_string())
                || event.tags.contains(&"defense_evasion".to_string())
                || event.tags.contains(&"process_injection".to_string())
                || event.tags.contains(&"auth_event".to_string())
                || event.tags.contains(&"script_exec".to_string())
            {
                if let Err(e) = event.validate_canonical_primitive() {
                    eprintln!(
                        "[capture_windows] VALIDATION ERROR: primitive event failed contract: {}",
                        e
                    );
                    eprintln!("[capture_windows] Event: {:?}", event);
                }
            }
        }

        // Write segment if events present
        if !all_events.is_empty() {
            let mut segment_file = fs::File::create(&segment_path)?;
            use std::io::Write;
            for evt in &all_events {
                if let Ok(json_str) = serde_json::to_string(evt) {
                    writeln!(segment_file, "{}", json_str)?;
                }
            }
        }

        self.event_count += all_events.len() as u64;

        // Rotate if needed
        if self.event_count >= self.config.segment_rotation_records {
            self.rotate_segment()?;
        }

        Ok(())
    }

    /// Assign EvidencePtr to events using capture writer state (segment seq + record counter)
    /// NEVER uses WEVTAPI record_id - that's internal Windows state not for evidence paths
    ///
    /// Guard assertion: incoming events must have evidence_ptr == None
    /// If any event already has Some(evidence_ptr), treat as architectural bug
    fn assign_evidence_ptrs_with_state(
        &self,
        mut events: Vec<crate::Event>,
        segment_seq: u64,
    ) -> Vec<crate::Event> {
        let mut record_index = 0u32;

        for event in &mut events {
            // GUARD: Incoming event must NOT have evidence_ptr already set
            if event.evidence_ptr.is_some() {
                debug_assert!(false, "[capture] BUG: incoming event already has evidence_ptr set - this violates architecture");
                eprintln!("[capture] ERROR: incoming event from collect already has evidence_ptr - dropping event as bug indicator");
                // Drop this event - it's a sign of architectural violation
                continue;
            }

            // Extract channel for stream_id
            let stream_id = event
                .fields
                .get("windows.channel")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            // Create EvidencePtr from capture writer state only
            event.evidence_ptr = Some(crate::core::EvidencePtr {
                stream_id,
                segment_id: segment_seq,
                record_index,
            });

            record_index = record_index.saturating_add(1);
        }

        // Filter out any events that triggered the guard (should be none in normal operation)
        events.retain(|e| e.evidence_ptr.is_some());

        events
    }

    /// Write heartbeat metrics for all sources
    pub fn write_heartbeat(&self) -> anyhow::Result<()> {
        for (source, metrics) in &self.metrics {
            let heartbeat = CaptureHeartbeat {
                ts: Utc::now(),
                transport: "evtx".to_string(),
                source: source.clone(),
                events_read_total: metrics.events_read_total,
                events_read_delta: metrics.events_read_delta,
                parse_failed_total: metrics.parse_failed_total,
                parse_failed_delta: metrics.parse_failed_delta,
                last_event_ts: metrics.last_event_ts,
                status: "healthy".to_string(),
                // Canonical event counters
                credential_access_count: self.cred_access_count,
                discovery_count: self.discovery_exec_count,
                exfiltration_count: self.archive_tool_exec_count + self.staging_write_count,
                network_connection_count: self.net_connect_prim_count,
                persistence_change_count: self.persistence_change_count,
                defense_evasion_count: self.defense_evasion_count,
                process_injection_count: self.proc_access_count,
                auth_event_count: self.priv_escalation_count,
                script_exec_count: self.script_exec_count,
                archive_tool_exec_count: self.archive_tool_exec_count,
                staging_write_count: self.staging_write_count,
            };

            let heartbeat_file = self
                .config
                .root_dir
                .join(format!("heartbeat_{}.json", source.replace("/", "_")));
            fs::write(&heartbeat_file, serde_json::to_string_pretty(&heartbeat)?)?;
        }

        Ok(())
    }

    /// Start capture loop
    pub fn run(&mut self) -> anyhow::Result<()> {
        // Spawn heartbeat thread
        let root = self.config.root_dir.clone();
        let interval = self.config.heartbeat_interval_secs;
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(interval));
            let heartbeat = serde_json::json!({
                "ts": Utc::now(),
                "transport": "evtx",
                "status": "running"
            });
            let hb_path = root.join("heartbeat.json");
            let _ = fs::write(hb_path, serde_json::to_string_pretty(&heartbeat).unwrap());
        });

        // Main capture loop
        loop {
            self.poll_and_write()?;
            self.write_heartbeat()?;
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn rotate_segment(&mut self) -> anyhow::Result<()> {
        self.segment_count += 1;
        self.event_count = 0;
        Ok(())
    }

    /// Write heartbeat with attack surface counters and primitive counters
    pub fn write_heartbeat(&self) -> anyhow::Result<()> {
        let heartbeat = serde_json::json!({
            "ts": Utc::now(),
            "pid": std::process::id(),
            "segment_id": self.segment_count,
            "schema_version": 1,
            "transport": "wevtapi",
            "proc_exec_count": self.proc_exec_count,
            "priv_escalation_count": self.priv_escalation_count,
            "proc_access_count": self.proc_access_count,
            "asr_block_count": self.asr_block_count,
            "wmi_persistence_count": self.wmi_persistence_count,
            "persistence_service_count": self.persistence_service_count,
            "persistence_task_count": self.persistence_task_count,
            "log_clear_count": self.log_clear_count,
            "remote_logon_rdp_count": self.remote_logon_rdp_count,
            "remote_winrm_count": self.remote_winrm_count,
            "decode_failed_count": self.decode_failed_count,
            "cred_access_count": self.cred_access_count,
            "discovery_exec_count": self.discovery_exec_count,
            "archive_tool_exec_count": self.archive_tool_exec_count,
            "staging_write_count": self.staging_write_count,
            "net_connect_prim_count": self.net_connect_prim_count,
            "persistence_change_count": self.persistence_change_count,
            "defense_evasion_count": self.defense_evasion_count,
            "script_exec_count": self.script_exec_count,
        });
        let hb_path = self.config.root_dir.join("capture_heartbeat.json");
        let temp_path = self.config.root_dir.join("capture_heartbeat.json.tmp");
        fs::write(&temp_path, serde_json::to_string_pretty(&heartbeat)?)?;
        fs::rename(&temp_path, &hb_path)?;
        Ok(())
    }
}

/// Sort events deterministically
fn sort_events(events: &mut Vec<crate::Event>) {
    events.sort_by(|a, b| {
        // Sort by ts_ms primarily, then by event_kind, then by pid
        match a.ts_ms.cmp(&b.ts_ms) {
            std::cmp::Ordering::Equal => {
                let a_kind = a
                    .fields
                    .get("event_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let b_kind = b
                    .fields
                    .get("event_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                match a_kind.cmp(b_kind) {
                    std::cmp::Ordering::Equal => {
                        let a_pid = a.fields.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                        let b_pid = b.fields.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                        a_pid.cmp(&b_pid)
                    }
                    other => other,
                }
            }
            other => other,
        }
    });
}

/// Generate dedup key for derived primitive events
/// Key is: event_kind + ts_ms + pid + (discriminator field for event type)
fn generate_dedup_key(event: &crate::Event) -> String {
    let kind = event
        .fields
        .get("event_kind")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let ts = event.ts_ms;
    let pid = event
        .fields
        .get("pid")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Add type-specific discriminators for all 9 canonical types
    let discriminator = if event.tags.contains(&"credential_access".to_string()) {
        event
            .fields
            .get("target_user")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else if event.tags.contains(&"discovery".to_string()) {
        event
            .fields
            .get("discovery_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else if event.tags.contains(&"exfiltration".to_string()) {
        event
            .fields
            .get(crate::core::event_keys::FILE_PATH)
            .and_then(|v| v.as_str())
            .unwrap_or(
                event
                    .fields
                    .get(crate::core::event_keys::ARCHIVE_TOOL)
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            )
    } else if event.tags.contains(&"network_connection".to_string()) {
        event
            .fields
            .get("remote_ip")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else if event.tags.contains(&"persistence_change".to_string()) {
        event
            .fields
            .get(crate::core::event_keys::PERSIST_LOCATION)
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else if event.tags.contains(&"defense_evasion".to_string()) {
        event
            .fields
            .get(crate::core::event_keys::EVASION_TARGET)
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else if event.tags.contains(&"process_injection".to_string()) {
        event
            .fields
            .get(crate::core::event_keys::INJECT_METHOD)
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else if event.tags.contains(&"auth_event".to_string()) {
        event
            .fields
            .get(crate::core::event_keys::AUTH_USER)
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else if event.tags.contains(&"script_exec".to_string()) {
        event
            .fields
            .get(crate::core::event_keys::SCRIPT_INTERPRETER)
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else {
        ""
    };

    format!("{}:{}:{}:{}", kind, ts, pid, discriminator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_new() {
        let tmpdir = std::env::temp_dir();
        let capture = WindowsEventCapture::new(tmpdir);
        assert_eq!(capture.segment_count, 0);
    }
}
