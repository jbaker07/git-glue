use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    Execve,
    Fexecve,
    Mmap,
    Mprotect,
    MemfdCreate,
    MemfdWrite,
    RemoteMap,
    ProcHollow,
    DllInject,
    Setuid,
    Capset,
    NonPrivFlip,
    SeccompRelax,
    Ptrace, // ← added to support mahalanobis.rs vectorizer
    KernelCfgFlip,
    PtDiff,
    RetainedFdPostExec,
    PipeCreate,
    FileToPipeBytes,
    PipeToSocketBytes,
    EpollWait,
    BeaconTick,
    NewAsn,
    Ja3Novelty,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SockMeta {
    pub family: Option<u16>, // AF_INET, AF_INET6, etc.
    pub laddr: Option<String>,
    pub lport: Option<u16>,
    pub raddr: Option<String>,
    pub rport: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePtr {
    pub stream_id: String,     // e.g., "evtx:Security", "sysmon:default"
    pub segment_id: String,    // e.g., "seg_12345"
    pub record_index: u64,     // Position in segment
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts: DateTime<Utc>,
    pub event_type: EventType,

    // Process identity
    pub pid: Option<i32>,
    pub ppid: Option<i32>,
    pub uid: Option<u32>,
    pub exe: Option<String>,
    pub argv: Option<Vec<String>>,
    pub cwd: Option<String>,

    // Resources / causality hooks
    pub fd: Option<i32>,
    pub inode_dev: Option<u64>,
    pub inode_ino: Option<u64>,
    pub bytes: Option<u64>,
    pub prot: Option<String>, // e.g., "RWX", "PROT_EXEC"
    pub sock: Option<SockMeta>,

    // Entity keys for correlation
    pub proc_key: Option<String>,       // pid@hostname or exe_hash
    pub file_key: Option<String>,       // inode or path_hash
    pub identity_key: Option<String>,   // user@domain or uid

    // Evidence tracking
    pub evidence_ptr: Option<EvidencePtr>,

    // Lightweight labels
    pub tags: Vec<String>,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            ts: Utc::now(),
            event_type: EventType::Other,
            pid: None,
            ppid: None,
            uid: None,
            exe: None,
            argv: None,
            cwd: None,
            fd: None,
            inode_dev: None,
            inode_ino: None,
            bytes: None,
            prot: None,
            sock: None,
            proc_key: None,
            file_key: None,
            identity_key: None,
            evidence_ptr: None,
            tags: Vec::new(),
        }
    }
}
