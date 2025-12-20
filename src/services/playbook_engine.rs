/// YAML-driven playbook engine for macOS threat detection
///
/// Schema: Each playbook YAML defines:
/// - playbook_id, title, severity
/// - entity_scope (how to group incidents: host|user|exe|session)
/// - slots (required + optional, with TTLs and match predicates)
/// - fire condition (which slots required + narrative)
///
/// Supports deterministic slot matching with TTL-based expiration.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

/// Predicate for matching a fact against a slot
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MatchPredicate {
    /// Exact match on a field value
    Equals {
        #[serde(rename = "event_type")]
        event_type: Option<Vec<String>>,
        #[serde(rename = "exe")]
        exe: Option<String>,
        #[serde(rename = "path")]
        path: Option<String>,
        #[serde(rename = "dst_port")]
        dst_port: Option<u16>,
    },
    /// Regex match on path or exe
    Regex {
        #[serde(rename = "path_regex")]
        path_regex: Option<String>,
        #[serde(rename = "exe_regex")]
        exe_regex: Option<String>,
    },
    /// Glob pattern match
    Glob {
        #[serde(rename = "path_glob")]
        path_glob: Option<String>,
    },
    /// Set membership (for event_type or uid)
    In {
        #[serde(rename = "event_type")]
        event_type: Option<Vec<String>>,
        #[serde(rename = "uid")]
        uid: Option<Vec<u32>>,
    },
    /// Numeric comparison
    Numeric {
        #[serde(rename = "dst_port")]
        dst_port: Option<NumericComparison>,
        #[serde(rename = "uid")]
        uid: Option<NumericComparison>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NumericComparison {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<i32>,
}

/// A slot definition (match requirement for a playbook)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlotDef {
    pub slot_id: String,
    pub required: bool,
    pub ttl_seconds: u64,
    #[serde(flatten)]
    pub predicate: MatchPredicate,
}

/// Fire condition (which slots must be matched to trigger incident)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FireDef {
    pub required_slots: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative: Option<String>,
}

/// Full YAML playbook definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaybookDef {
    #[serde(default = "default_playbook_schema_version")]
    pub playbook_schema_version: u32,
    pub playbook_id: String,
    pub title: String,
    pub severity: String,
    #[serde(default = "default_entity_scope")]
    pub entity_scope: String,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub slots: Vec<SlotDef>,
    pub fire: FireDef,
    #[serde(skip)]
    pub playbook_hash: String,
    #[serde(skip)]
    pub playbook_source_relpath: String,
}

fn default_playbook_schema_version() -> u32 {
    1
}

fn default_entity_scope() -> String {
    "host|user|exe".to_string()
}

fn default_priority() -> u32 {
    100 // Default priority (higher = more important)
}

/// Tracked slot fill (event that matched the slot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotFill {
    pub slot_id: String,
    pub ts_ms: u64,
    pub fact_type: String,
    pub exe: String,
    pub path: Option<String>,
    pub evidence_ptr: Option<String>, // segment_id:record_index
}

/// Per-entity playbook state
#[derive(Debug, Clone)]
pub struct PlaybookState {
    pub filled_slots: HashMap<String, SlotFill>,
    pub first_ts_ms: u64,
    pub last_ts_ms: u64,
    pub expires_ts_ms: u64,
}

impl PlaybookState {
    fn new(now_ms: u64, ttl_ms: u64) -> Self {
        Self {
            filled_slots: HashMap::new(),
            first_ts_ms: now_ms,
            last_ts_ms: now_ms,
            expires_ts_ms: now_ms + ttl_ms,
        }
    }

    fn is_expired(&self, now_ms: u64) -> bool {
        now_ms > self.expires_ts_ms
    }
}

/// YAML playbook engine
pub struct PlaybookEngine {
    pub playbooks: Vec<PlaybookDef>,
    states: HashMap<String, HashMap<String, PlaybookState>>, // playbook_id -> entity_key -> state
    global_ttl_ms: u64,
}

impl PlaybookEngine {
    pub fn new(playbooks: Vec<PlaybookDef>, global_ttl_ms: u64) -> Self {
        Self {
            playbooks,
            states: HashMap::new(),
            global_ttl_ms,
        }
    }
    
    /// Compute SHA256 hash of a playbook (deterministic JSON canonical form)
    pub fn compute_playbook_hash(playbook: &PlaybookDef) -> String {
        use sha2::{Sha256, Digest};
        // Use serde_json to create canonical form (sorted keys, no extra spaces)
        let json_str = serde_json::to_string(playbook).unwrap_or_else(|_| format!("{:?}", playbook.playbook_id));
        let mut hasher = Sha256::new();
        hasher.update(json_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Try to match a fact against all loaded playbooks
    /// Returns vec of (playbook_id, entity_key, incident_ready) if any slots matched
    pub fn match_fact(
        &mut self,
        fact_type: &str,
        exe: &str,
        path: Option<&str>,
        dst_port: Option<u16>,
        uid: Option<u32>,
        user: &str,
        host: &str,
        event_type: &str,
        ts_ms: u64,
        evidence_ptr: Option<&str>,
    ) -> Vec<(String, String, bool)> {
        let now_ms = current_unix_ms();
        let mut results = Vec::new();

        // Clone playbooks to avoid borrow issues
        let playbooks = self.playbooks.clone();

        for playbook in &playbooks {
            // Clean up expired states
            let playbook_states = self.states.entry(playbook.playbook_id.clone())
                .or_insert_with(HashMap::new);
            playbook_states.retain(|_, state| !state.is_expired(now_ms));

            // Compute entity key based on scope
            let entity_key = self.compute_entity_key(&playbook.entity_scope, user, host, exe);

            // Try to match each slot
            for slot in &playbook.slots {
                if self.predicate_matches(slot, fact_type, exe, path, dst_port, uid, event_type) {
                    // Matched this slot
                    let filled_slots = {
                        let state = self.states
                            .entry(playbook.playbook_id.clone())
                            .or_insert_with(HashMap::new)
                            .entry(entity_key.clone())
                            .or_insert_with(|| PlaybookState::new(now_ms, self.global_ttl_ms));

                        state.filled_slots.insert(slot.slot_id.clone(), SlotFill {
                            slot_id: slot.slot_id.clone(),
                            ts_ms,
                            fact_type: fact_type.to_string(),
                            exe: exe.to_string(),
                            path: path.map(|p| p.to_string()),
                            evidence_ptr: evidence_ptr.map(|p| p.to_string()),
                        });

                        state.last_ts_ms = ts_ms;
                        state.filled_slots.clone()  // Clone for fire check
                    };

                    // Check if playbook is ready to fire
                    let fire_ready = self.check_fire_ready(&playbook.fire, &filled_slots);
                    if fire_ready {
                        results.push((playbook.playbook_id.clone(), entity_key.clone(), true));
                    } else {
                        results.push((playbook.playbook_id.clone(), entity_key.clone(), false));
                    }
                }
            }
        }

        results
    }

    pub fn match_signal_fact(
        &self,
        signal_type: &str,
        severity: &str,
        host: &str,
        user: &str,
        exe: Option<&str>,
        _ts_ms: u64,
    ) -> Vec<(String, String, bool)> {
        let mut results = Vec::new();
        
        for playbook in &self.playbooks {
            for slot in &playbook.slots {
                if slot.kind == "signal" {
                    if let Some(match_spec) = &slot.match_spec {
                        if let Some(sig_type) = match_spec.get("signal_type").and_then(|v| v.as_str()) {
                            if sig_type == signal_type {
                                results.push((playbook.playbook_id.clone(), slot.slot_id.clone(), true));
                            }
                        }
                    }
                }
            }
        }
        results
    }

    fn compute_entity_key(&self, scope: &str, user: &str, host: &str, exe: &str) -> String {
        // Entity scope components: user|host|exe|session
        let parts: Vec<&str> = scope.split('|').collect();
        let mut key_parts = Vec::new();

        for part in parts {
            match part {
                "user" => key_parts.push(user.to_string()),
                "host" => key_parts.push(host.to_string()),
                "exe" => key_parts.push(exe.to_string()),
                _ => {}
            }
        }

        key_parts.join("|")
    }

    fn predicate_matches(
        &self,
        slot: &SlotDef,
        _fact_type: &str,
        exe: &str,
        _path: Option<&str>,
        dst_port: Option<u16>,
        uid: Option<u32>,
        event_type: &str,
    ) -> bool {
        match &slot.predicate {
            MatchPredicate::Equals {
                event_type: et,
                exe: expected_exe,
                path: expected_path,
                dst_port: expected_port,
            } => {
                // All specified fields must match
                if let Some(et_list) = et {
                    if !et_list.contains(&event_type.to_string()) {
                        return false;
                    }
                }
                if let Some(exp_exe) = expected_exe {
                    if exp_exe != exe {
                        return false;
                    }
                }
                if let Some(exp_path) = expected_path {
                    if path != Some(exp_path.as_str()) {
                        return false;
                    }
                }
                if let Some(exp_port) = expected_port {
                    if dst_port != Some(*exp_port) {
                        return false;
                    }
                }
                true
            }
            MatchPredicate::Regex {
                path_regex,
                exe_regex,
            } => {
                // Regex matching
                if let Some(regex_str) = exe_regex {
                    if let Ok(re) = regex::Regex::new(regex_str) {
                        if !re.is_match(exe) {
                            return false;
                        }
                    }
                }
                if let Some(regex_str) = path_regex {
                    if let Some(p) = path {
                        if let Ok(re) = regex::Regex::new(regex_str) {
                            if !re.is_match(p) {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            MatchPredicate::Glob { path_glob } => {
                // Glob matching (simplified)
                if let Some(glob_pat) = path_glob {
                    if let Some(p) = path {
                        return glob_pat_matches(glob_pat, p);
                    }
                }
                true
            }
            MatchPredicate::In { event_type: et, uid: uid_list } => {
                if let Some(et_list) = et {
                    if !et_list.contains(&event_type.to_string()) {
                        return false;
                    }
                }
                if let Some(uid_set) = uid_list {
                    if let Some(u) = uid {
                        if !uid_set.contains(&u) {
                            return false;
                        }
                    }
                }
                true
            }
            MatchPredicate::Numeric { dst_port: port_cmp, uid: uid_cmp } => {
                if let Some(port_comp) = port_cmp {
                    if let Some(port) = dst_port {
                        if !numeric_comparison_match(port as i32, port_comp) {
                            return false;
                        }
                    }
                }
                if let Some(uid_comp) = uid_cmp {
                    if let Some(u) = uid {
                        if !numeric_comparison_match(u as i32, uid_comp) {
                            return false;
                        }
                    }
                }
                true
            }
        }
    }

    fn check_fire_ready(&self, fire_def: &FireDef, filled_slots: &HashMap<String, SlotFill>) -> bool {
        // Check if all required slots are filled
        for required in &fire_def.required_slots {
            if !filled_slots.contains_key(required) {
                return false;
            }
        }
        true
    }

    /// Get current state for a playbook + entity
    pub fn get_state(&self, playbook_id: &str, entity_key: &str) -> Option<PlaybookState> {
        self.states
            .get(playbook_id)
            .and_then(|states| states.get(entity_key).cloned())
    }
}

fn glob_pat_matches(pattern: &str, text: &str) -> bool {
    // Simple glob support: * = any, ? = single char
    let pat_bytes = pattern.as_bytes();
    let text_bytes = text.as_bytes();
    let mut pi = 0;
    let mut ti = 0;

    while pi < pat_bytes.len() && ti < text_bytes.len() {
        match pat_bytes[pi] {
            b'*' => {
                // Greedy match
                pi += 1;
                if pi >= pat_bytes.len() {
                    return true; // * at end matches rest
                }
                // Find next match
                while ti <= text_bytes.len() {
                    if glob_pat_matches(&pattern[pi..], &text[ti..]) {
                        return true;
                    }
                    ti += 1;
                }
                return false;
            }
            b'?' => {
                pi += 1;
                ti += 1;
            }
            c if c == pat_bytes[pi] => {
                pi += 1;
                ti += 1;
            }
            _ => return false,
        }
    }

    pi == pat_bytes.len() && ti == text_bytes.len()
}

fn numeric_comparison_match(value: i32, cmp: &NumericComparison) -> bool {
    if let Some(lt) = cmp.lt {
        if value >= lt {
            return false;
        }
    }
    if let Some(lte) = cmp.lte {
        if value > lte {
            return false;
        }
    }
    if let Some(gt) = cmp.gt {
        if value <= gt {
            return false;
        }
    }
    if let Some(gte) = cmp.gte {
        if value < gte {
            return false;
        }
    }
    true
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn load_playbooks_from_dir(playbooks_dir: &PathBuf) -> Result<Vec<PlaybookDef>, String> {
    if !playbooks_dir.exists() {
        return Ok(Vec::new()); // No playbooks directory
    }

    let mut playbooks = Vec::new();

    for entry in std::fs::read_dir(playbooks_dir)
        .map_err(|e| format!("Failed to read playbooks dir: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let path = entry.path();

        if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            let yaml_str = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            
            // Compute SHA-256 hash of canonical YAML bytes (fail-fast if unreadable)
            let playbook_hash = {
                let mut hasher = Sha256::new();
                hasher.update(yaml_str.as_bytes());
                format!("{:x}", hasher.finalize())
            };
            
            let mut playbook: PlaybookDef = serde_yaml::from_str(&yaml_str)
                .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
            
            // Populate hash and source path
            playbook.playbook_hash = playbook_hash;
            playbook.playbook_source_relpath = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            // Validate playbook structure
            validate_playbook(&playbook, &path)
                .map_err(|e| format!("Validation error in {}: {}", path.display(), e))?;
            
            playbooks.push(playbook);
        }
    }

    Ok(playbooks)
}

fn validate_playbook(pb: &PlaybookDef, path: &PathBuf) -> Result<(), String> {
    // Check schema version
    if pb.playbook_schema_version != 1 {
        return Err(format!("unsupported playbook_schema_version: {} (expected 1)", pb.playbook_schema_version));
    }
    
    // Check required fields
    if pb.playbook_id.is_empty() {
        return Err("playbook_id is empty".to_string());
    }
    if pb.title.is_empty() {
        return Err("title is empty".to_string());
    }
    if pb.severity.is_empty() {
        return Err("severity is empty".to_string());
    }
    
    // Check slots exist and are unique
    if pb.slots.is_empty() {
        return Err("playbook has no slots defined".to_string());
    }
    
    let mut slot_ids = HashSet::new();
    for slot in &pb.slots {
        if slot.slot_id.is_empty() {
            return Err("slot_id is empty".to_string());
        }
        if slot_ids.contains(&slot.slot_id) {
            return Err(format!("duplicate slot_id: {}", slot.slot_id));
        }
        slot_ids.insert(slot.slot_id.clone());
        
        // Check TTL > 0
        if slot.ttl_seconds == 0 {
            return Err(format!("slot {} has ttl_seconds = 0 (must be > 0)", slot.slot_id));
        }
    }
    
    // Check fire condition references valid slots
    for required_slot in &pb.fire.required_slots {
        if !slot_ids.contains(required_slot) {
            return Err(format!("fire.required_slots references unknown slot: {}", required_slot));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_key_computation() {
        let engine = PlaybookEngine::new(vec![], 60_000);
        let key = engine.compute_entity_key("user|host|exe", "alice", "myhost", "/bin/ls");
        assert_eq!(key, "alice|myhost|/bin/ls");
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_pat_matches("/tmp/*", "/tmp/malware.sh"));
        assert!(!glob_pat_matches("/tmp/*", "/var/tmp/malware.sh"));
        assert!(glob_pat_matches("/var/tmp/*.plist", "/var/tmp/com.test.plist"));
    }

    #[test]
    fn test_numeric_comparison() {
        let cmp = NumericComparison {
            gte: Some(1024),
            lte: Some(65535),
            lt: None,
            gt: None,
        };
        assert!(numeric_comparison_match(8080, &cmp));
        assert!(!numeric_comparison_match(512, &cmp));
        assert!(!numeric_comparison_match(70000, &cmp));
    }
}
