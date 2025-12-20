/// Deterministic incident ID generation
/// Incident IDs must be stable across runs for the same input facts

use sha2::{Sha256, Digest};

/// Generate a stable incident ID from playbook + entity + first timestamp
/// 
/// Format: hash(playbook_id + primary_entity + ts_bucket)
/// This ensures the same logical incident (playbook fired for same entity in same window) 
/// always gets the same ID, even on restart.
pub fn compute_incident_id(
    playbook_id: &str,
    primary_entity: &str,  // e.g., "host:user:exe"
    first_slot_ts: u64,    // unix seconds
) -> String {
    // Round ts to 1-hour bucket for stability (same hour = same incident even if events spread)
    let ts_bucket = (first_slot_ts / 3600) * 3600;
    
    let key = format!("{}|{}|{}", playbook_id, primary_entity, ts_bucket);
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let hash = hasher.finalize();
    
    // Use first 12 hex chars for readability (still ~48 bits of entropy)
    format!("{:x}", hash)[..12].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let id1 = compute_incident_id("pb_ransomware", "host1:user1:exe1", 1702900000);
        let id2 = compute_incident_id("pb_ransomware", "host1:user1:exe1", 1702900000);
        assert_eq!(id1, id2, "Same input should produce same ID");
    }

    #[test]
    fn test_different_playbooks() {
        let id1 = compute_incident_id("pb_ransomware", "host1:user1:exe1", 1702900000);
        let id2 = compute_incident_id("pb_lateral", "host1:user1:exe1", 1702900000);
        assert_ne!(id1, id2, "Different playbooks should produce different IDs");
    }
}
