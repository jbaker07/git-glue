/// macOS production detection engine using OpenBSM facts
/// Spec: §0-9 - Deterministic, idempotent, evidence-driven
///
/// Non-negotiables:
/// - Every incident includes evidence_ptrs to facts
/// - Deterministic dedupe_keys prevent duplicates
/// - Reprocessing same segments = same incidents (idempotent)
/// - All timestamps comparable and deterministic

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

pub mod enricher;
pub mod facts;
pub mod incidents;
pub mod playbooks;
pub mod state_machine;
pub mod explain;
pub mod narrative;
pub mod explainer;
pub mod api;
pub mod db_schema;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod synthetic_pipeline_tests;

// ==================== Correlation Primitives ====================

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionKey {
    pub host: String,
    pub audit_session_id: Option<u32>,
    pub auid: Option<u32>,
    pub uid: Option<u32>,
}

impl SessionKey {
    pub fn new(host: String, auid: Option<u32>, uid: Option<u32>, sid: Option<u32>) -> Self {
        Self {
            host,
            audit_session_id: sid,
            auid,
            uid,
        }
    }

    pub fn deterministic_hash(&self) -> String {
        let key_str = format!(
            "{}:{}:{}:{}",
            self.host,
            self.audit_session_id.unwrap_or(0),
            self.auid.unwrap_or(0),
            self.uid.unwrap_or(0)
        );
        format!("{:x}", sha2_hash(&key_str))
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProcKey {
    pub host: String,
    pub pid: u32,
    pub start_ts: u64, // Unix ms
}

impl ProcKey {
    pub fn new(host: String, pid: u32, start_ts: u64) -> Self {
        Self { host, pid, start_ts }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileKey {
    pub canonical_path: String,
    pub inode: Option<u64>,
    pub mtime: Option<u64>,
}

impl FileKey {
    pub fn new(path: String, inode: Option<u64>, mtime: Option<u64>) -> Self {
        Self {
            canonical_path: path,
            inode,
            mtime,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NetKey {
    pub host: String,
    pub dest_ip: String,
    pub dest_port: u16,
    pub protocol: String, // tcp, udp
    pub proc_key_hash: Option<String>,
}

// ==================== Fact Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub fact_id: String,          // deterministic within segment
    pub ts: u64,                  // Unix ms, monotonic
    pub host: String,
    pub uid: Option<u32>,
    pub euid: Option<u32>,
    pub gid: Option<u32>,
    pub auid: Option<u32>,        // audit user id
    pub audit_session_id: Option<u32>,
    pub pid: Option<u32>,
    pub ppid: Option<u32>,
    pub exe_path: Option<String>,
    pub argv: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub fact_type: FactType,
    pub segment_id: String,       // source segment for evidence
}

impl Fact {
    pub fn subject_key(&self) -> SessionKey {
        SessionKey::new(
            self.host.clone(),
            self.auid,
            self.uid,
            self.audit_session_id,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FactType {
    ProcExec(ProcExecFact),
    FileWrite(FileOpFact),
    FileRename(FileOpFact),
    FileUnlink(FileOpFact),
    FileChmod(FileOpFact),
    FileChown(FileOpFact),
    NetConnect(NetOpFact),
    NetListen(NetOpFact),
    PrivilegeChange(PrivilegeChangeFact),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcExecFact {
    pub exe_path: String,
    pub argv: Vec<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub parent_exe: Option<String>,
    pub start_ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOpFact {
    pub path: String,
    pub inode: Option<u64>,
    pub mtime: Option<u64>,
    pub size: Option<u64>,
    pub mode: Option<u32>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetOpFact {
    pub src_ip: Option<String>,
    pub src_port: Option<u16>,
    pub dest_ip: String,
    pub dest_port: u16,
    pub protocol: String, // tcp, udp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeChangeFact {
    pub new_euid: u32,
    pub new_gid: u32,
    pub old_euid: Option<u32>,
    pub old_gid: Option<u32>,
}

// ==================== Enriched Fact ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedFact {
    pub fact: Fact,
    pub enrichments: Enrichments,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Enrichments {
    pub canonical_path: Option<String>,
    pub canonical_argv: Option<Vec<String>>,
    pub codesign: Option<CodesignInfo>,
    pub rarity: Option<RarityFlags>,
    pub content: Option<ContentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodesignInfo {
    pub signed: Option<bool>,
    pub apple_signed: Option<bool>,
    pub team_id: Option<String>,
    pub identifier: Option<String>,
    pub notarized: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityFlags {
    pub first_seen_host: bool,
    pub first_seen_user: bool,
    pub first_seen_tuple: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentInfo {
    pub size_bytes: u64,
    pub entropy: Option<f64>,
    pub base64_score: Option<f64>,
    pub sha256: Option<String>,
}

// ==================== Incident (Detection Result) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,                     // UUID or stable hash
    pub dedupe_key: String,             // deterministic, unique index candidate
    pub playbook_id: String,            // A, B, C, D, E, F
    pub severity: Severity,
    pub confidence: f64,                // 0.0 - 1.0
    pub host: String,
    pub uid: Option<u32>,
    pub session_key_hash: String,
    pub exe_path: Option<String>,
    pub mitre_tags: Vec<String>,
    pub summary: String,
    pub first_seen_ts: u64,
    pub last_seen_ts: u64,
    pub window_start_ts: u64,
    pub window_end_ts: u64,
    pub evidence_ptrs: Vec<EvidencePtr>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<crate::macos_detect::explain::ExplanationBundle>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum Severity {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePtr {
    pub segment_id: String,
    pub fact_id: String,
    pub fact_type: String,
    pub ts: u64,
}

// ==================== State & Dedup ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookState {
    pub session_key_hash: String,
    pub matched_steps: HashSet<String>,
    pub evidence_ptrs: Vec<EvidencePtr>,
    pub first_event_ts: u64,
    pub last_event_ts: u64,
    pub created_ts: u64,
    pub updated_ts: u64,
    pub expires_ts: u64,
    pub step_data: HashMap<String, serde_json::Value>,
}

// ==================== Helpers ====================

pub fn sha2_hash(input: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let bytes: [u8; 8] = result[0..8].try_into().unwrap_or([0; 8]);
    u64::from_le_bytes(bytes)
}

pub fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn hour_bucket(ts: u64) -> u64 {
    (ts / 3_600_000) * 3_600_000
}

pub fn is_user_writable_dir(path: &str) -> bool {
    let suspicious = [
        "/tmp",
        "/private/tmp",
        "/var/tmp",
        "/Users/Shared",
        "~/Downloads",
        "~/Desktop",
        "~/Library",
        "~/Documents",
    ];
    suspicious.iter().any(|s| path.contains(s))
}

pub fn is_launchd_path(path: &str) -> bool {
    path.contains("LaunchAgents") || path.contains("LaunchDaemons")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_determinism() {
        let key1 = SessionKey::new("host1".into(), Some(1000), Some(501), Some(42));
        let key2 = SessionKey::new("host1".into(), Some(1000), Some(501), Some(42));
        assert_eq!(key1.deterministic_hash(), key2.deterministic_hash());
    }

    #[test]
    fn test_hour_bucket() {
        let ts1 = 1_000_000;
        let ts2 = 1_500_000;
        assert_eq!(hour_bucket(ts1), hour_bucket(ts2)); // same hour
        assert_ne!(hour_bucket(ts1), hour_bucket(ts1 + 3_600_001));
    }
}
