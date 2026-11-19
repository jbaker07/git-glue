/// retention_contract.rs: Evidence retention contract enforcement
/// 
/// Defines and enforces evidence retention requirements:
/// - Minimum retention window (P0): Cannot delete evidence older than minimum window
/// - Maximum retention window (P1): Must delete evidence older than maximum window
/// - Graceful fallback: On quota exceeded, prefer keeping recent evidence
///
/// This ensures compliance with legal holds, investigation windows, and storage constraints.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Minimum retention: Never delete evidence older than this many milliseconds from now
    /// Default: 30 days (30 * 24 * 60 * 60 * 1000 ms)
    pub min_retention_ms: u64,
    
    /// Maximum retention: Must delete evidence older than this many milliseconds from now
    /// Default: 365 days (1 year)
    pub max_retention_ms: u64,
    
    /// Maximum storage quota in bytes before enforcement
    /// Default: 1 TB
    pub max_storage_bytes: u64,
    
    /// Enforcement mode: "strict" requires deletion, "graceful" logs warnings
    pub enforcement_mode: String,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        RetentionPolicy {
            min_retention_ms: 30 * 24 * 60 * 60 * 1000,      // 30 days
            max_retention_ms: 365 * 24 * 60 * 60 * 1000,     // 1 year
            max_storage_bytes: 1024 * 1024 * 1024 * 1024,    // 1 TB
            enforcement_mode: "strict".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionContract {
    pub policy: RetentionPolicy,
    pub contract_id: String,
    pub created_ts: u64,
    pub enforced_ts: Option<u64>,
    pub violation_count: u32,
    pub deleted_bytes: u64,
}

impl RetentionContract {
    pub fn new(policy: RetentionPolicy) -> Self {
        let contract_id = format!("contract_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
        RetentionContract {
            policy,
            contract_id,
            created_ts: now_ms(),
            enforced_ts: None,
            violation_count: 0,
            deleted_bytes: 0,
        }
    }
    
    /// Check if a segment (by ts_ms) is within minimum retention window (cannot delete)
    pub fn is_within_min_retention(&self, segment_ts_ms: u64) -> bool {
        let now = now_ms();
        let age_ms = now.saturating_sub(segment_ts_ms);
        age_ms < self.policy.min_retention_ms
    }
    
    /// Check if a segment (by ts_ms) is beyond maximum retention window (must delete)
    pub fn is_beyond_max_retention(&self, segment_ts_ms: u64) -> bool {
        let now = now_ms();
        let age_ms = now.saturating_sub(segment_ts_ms);
        age_ms > self.policy.max_retention_ms
    }
    
    /// Check if storage quota is exceeded
    pub fn is_quota_exceeded(&self, current_bytes: u64) -> bool {
        current_bytes > self.policy.max_storage_bytes
    }
    
    /// Determine segments eligible for deletion (beyond max retention)
    pub fn segments_to_delete(&self, segments: &[SegmentDeleteInfo]) -> Vec<String> {
        segments
            .iter()
            .filter(|seg| self.is_beyond_max_retention(seg.ts_last_ms))
            .map(|seg| seg.segment_id.clone())
            .collect()
    }
    
    /// Graceful deletion: when quota exceeded, delete oldest segments outside min_retention window
    pub fn graceful_delete_plan(
        &self,
        segments: &[SegmentDeleteInfo],
        current_bytes: u64,
        target_bytes: u64,
    ) -> GracefulDeletePlan {
        let mut plan = GracefulDeletePlan {
            segments_to_delete: Vec::new(),
            bytes_to_free: 0,
            reason: "quota_not_exceeded".to_string(),
        };
        
        if !self.is_quota_exceeded(current_bytes) {
            return plan;
        }
        
        plan.reason = "quota_exceeded".to_string();
        let bytes_to_free = current_bytes.saturating_sub(target_bytes);
        plan.bytes_to_free = bytes_to_free;
        
        // Sort by time (oldest first), skip those in min_retention window
        let mut candidates: Vec<_> = segments
            .iter()
            .filter(|seg| !self.is_within_min_retention(seg.ts_last_ms))
            .collect();
        
        candidates.sort_by_key(|seg| seg.ts_last_ms);
        
        let mut freed = 0u64;
        for seg in candidates {
            if freed >= bytes_to_free {
                break;
            }
            plan.segments_to_delete.push(seg.segment_id.clone());
            freed += seg.size_bytes;
        }
        
        plan
    }
    
    /// Evaluate retention contract for a set of segments
    pub fn evaluate(
        &mut self,
        segments: &[SegmentDeleteInfo],
        current_bytes: u64,
    ) -> EvaluationResult {
        let mut result = EvaluationResult {
            contract_id: self.contract_id.clone(),
            policy_violations: Vec::new(),
            mandatory_deletions: Vec::new(),
            graceful_delete_plan: GracefulDeletePlan::default(),
            status: "compliant".to_string(),
        };
        
        // Check for mandatory deletions (beyond max_retention)
        for seg in segments {
            if self.is_beyond_max_retention(seg.ts_last_ms) {
                result.mandatory_deletions.push(seg.segment_id.clone());
                self.violation_count += 1;
            }
        }
        
        if !result.mandatory_deletions.is_empty() {
            result.status = "violation_found".to_string();
        }
        
        // Check for quota exceeded
        if self.is_quota_exceeded(current_bytes) {
            let target = (self.policy.max_storage_bytes * 9) / 10; // 90% of max
            result.graceful_delete_plan = self.graceful_delete_plan(segments, current_bytes, target);
            result.status = "quota_exceeded".to_string();
        }
        
        // Record enforcement timestamp
        if !result.mandatory_deletions.is_empty() || !result.graceful_delete_plan.segments_to_delete.is_empty() {
            self.enforced_ts = Some(now_ms());
        }
        
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDeleteInfo {
    pub segment_id: String,
    pub ts_first_ms: u64,
    pub ts_last_ms: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GracefulDeletePlan {
    pub segments_to_delete: Vec<String>,
    pub bytes_to_free: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub contract_id: String,
    pub policy_violations: Vec<String>,      // Human-readable violations
    pub mandatory_deletions: Vec<String>,    // Segments that MUST be deleted
    pub graceful_delete_plan: GracefulDeletePlan, // Suggested graceful deletions
    pub status: String,                      // "compliant", "violation_found", "quota_exceeded"
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_retention_window() {
        let policy = RetentionPolicy {
            min_retention_ms: 86400000,  // 1 day
            ..Default::default()
        };
        let contract = RetentionContract::new(policy);
        
        // Recent segment (within 1 day): should be protected
        let recent_ts = now_ms() - 3600000;  // 1 hour ago
        assert!(contract.is_within_min_retention(recent_ts));
        
        // Old segment (beyond min window): not protected
        let old_ts = now_ms() - (2 * 86400000);  // 2 days ago
        assert!(!contract.is_within_min_retention(old_ts));
    }

    #[test]
    fn test_max_retention_violation() {
        let policy = RetentionPolicy {
            max_retention_ms: 86400000,  // 1 day
            ..Default::default()
        };
        let contract = RetentionContract::new(policy);
        
        // Recent segment: should not trigger deletion
        let recent_ts = now_ms() - 3600000;  // 1 hour ago
        assert!(!contract.is_beyond_max_retention(recent_ts));
        
        // Very old segment: should trigger deletion
        let very_old_ts = now_ms() - (2 * 86400000);  // 2 days ago
        assert!(contract.is_beyond_max_retention(very_old_ts));
    }

    #[test]
    fn test_quota_exceeded() {
        let policy = RetentionPolicy {
            max_storage_bytes: 1000000,  // 1 MB
            ..Default::default()
        };
        let contract = RetentionContract::new(policy);
        
        assert!(!contract.is_quota_exceeded(500000));    // Under quota
        assert!(contract.is_quota_exceeded(1500000));   // Over quota
    }

    #[test]
    fn test_graceful_deletion_plan() {
        let policy = RetentionPolicy {
            min_retention_ms: 86400000,  // 1 day
            max_retention_ms: 365 * 86400000,  // 1 year
            max_storage_bytes: 1000000,  // 1 MB
            enforcement_mode: "graceful".to_string(),
        };
        let contract = RetentionContract::new(policy);
        
        let now = now_ms();
        let segments = vec![
            SegmentDeleteInfo {
                segment_id: "seg_old".to_string(),
                ts_first_ms: now - (2 * 86400000),
                ts_last_ms: now - (2 * 86400000),
                size_bytes: 500000,
            },
            SegmentDeleteInfo {
                segment_id: "seg_recent".to_string(),
                ts_first_ms: now - 3600000,
                ts_last_ms: now - 3600000,
                size_bytes: 600000,
            },
        ];
        
        let plan = contract.graceful_delete_plan(&segments, 1100000, 900000);
        assert!(plan.segments_to_delete.contains(&"seg_old".to_string()));
        assert!(!plan.segments_to_delete.contains(&"seg_recent".to_string()));
    }
}
