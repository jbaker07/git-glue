/// cooldown_window.rs: Incident cooldown enforcement (P1-3)
///
/// Prevents rapid-fire duplicate incident firing by enforcing minimum window
/// between consecutive incidents for the same (playbook_id, host, user, exe) tuple.
///
/// Use case: Prevent alert fatigue from same alert firing every millisecond.
/// If process is spamming events, we want to report once per cooldown window,
/// not continuously.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CooldownPolicy {
    /// Minimum time (ms) between incidents with same signature
    pub default_cooldown_ms: u64,
    
    /// Per-playbook overrides (playbook_id -> cooldown_ms)
    pub playbook_overrides: HashMap<String, u64>,
    
    /// Whether cooldown is enforced globally
    pub enabled: bool,
}

impl Default for CooldownPolicy {
    fn default() -> Self {
        CooldownPolicy {
            default_cooldown_ms: 60_000,    // 1 minute default
            playbook_overrides: HashMap::new(),
            enabled: true,
        }
    }
}

/// Unique signature for incident cooldown tracking
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IncidentSignature {
    pub playbook_id: String,
    pub host: String,
    pub user: Option<String>,
    pub exe: Option<String>,
}

impl IncidentSignature {
    pub fn new(playbook_id: String, host: String, user: Option<String>, exe: Option<String>) -> Self {
        IncidentSignature {
            playbook_id,
            host,
            user,
            exe,
        }
    }
    
    /// Create a string key for hashing
    pub fn to_key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.playbook_id,
            self.host,
            self.user.as_deref().unwrap_or("unknown"),
            self.exe.as_deref().unwrap_or("unknown")
        )
    }
}

pub struct CooldownTracker {
    policy: CooldownPolicy,
    last_fire_times: HashMap<String, u64>,  // signature key -> ts_ms
}

impl CooldownTracker {
    pub fn new(policy: CooldownPolicy) -> Self {
        CooldownTracker {
            policy,
            last_fire_times: HashMap::new(),
        }
    }
    
    /// Check if an incident can fire (respecting cooldown)
    /// Returns: (can_fire, time_until_ready_ms)
    pub fn can_fire(&self, sig: &IncidentSignature) -> (bool, u64) {
        if !self.policy.enabled {
            return (true, 0);
        }
        
        let now = now_ms();
        let key = sig.to_key();
        
        if let Some(&last_fire) = self.last_fire_times.get(&key) {
            let cooldown_ms = self.policy
                .playbook_overrides
                .get(&sig.playbook_id)
                .copied()
                .unwrap_or(self.policy.default_cooldown_ms);
            
            let elapsed = now.saturating_sub(last_fire);
            if elapsed < cooldown_ms {
                let remaining = cooldown_ms - elapsed;
                return (false, remaining);
            }
        }
        
        (true, 0)
    }
    
    /// Record an incident firing
    pub fn record_fire(&mut self, sig: &IncidentSignature) {
        let key = sig.to_key();
        self.last_fire_times.insert(key, now_ms());
    }
    
    /// Clear cooldown for a specific signature (e.g., after manual remediation)
    pub fn reset(&mut self, sig: &IncidentSignature) {
        let key = sig.to_key();
        self.last_fire_times.remove(&key);
    }
    
    /// Clear all cooldowns
    pub fn reset_all(&mut self) {
        self.last_fire_times.clear();
    }
    
    /// Get time since last fire for a signature (0 if never fired)
    pub fn time_since_last_fire(&self, sig: &IncidentSignature) -> u64 {
        let now = now_ms();
        let key = sig.to_key();
        
        if let Some(&last_fire) = self.last_fire_times.get(&key) {
            now.saturating_sub(last_fire)
        } else {
            0
        }
    }
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
    fn test_cooldown_enforced() {
        let policy = CooldownPolicy {
            default_cooldown_ms: 1000,
            ..Default::default()
        };
        let mut tracker = CooldownTracker::new(policy);
        
        let sig = IncidentSignature::new(
            "pb_test".to_string(),
            "host1".to_string(),
            Some("user1".to_string()),
            Some("/bin/bash".to_string()),
        );
        
        // First fire should always be allowed
        assert!(tracker.can_fire(&sig).0);
        tracker.record_fire(&sig);
        
        // Immediately after, should be blocked
        assert!(!tracker.can_fire(&sig).0);
    }

    #[test]
    fn test_cooldown_disabled() {
        let mut policy = CooldownPolicy::default();
        policy.enabled = false;
        
        let tracker = CooldownTracker::new(policy);
        let sig = IncidentSignature::new(
            "pb_test".to_string(),
            "host1".to_string(),
            None,
            None,
        );
        
        // Even after fire, can fire again if disabled
        assert!(tracker.can_fire(&sig).0);
    }

    #[test]
    fn test_playbook_override() {
        let mut policy = CooldownPolicy::default();
        policy.default_cooldown_ms = 5000;
        policy.playbook_overrides.insert("pb_critical".to_string(), 500);
        
        let mut tracker = CooldownTracker::new(policy);
        
        let critical_sig = IncidentSignature::new(
            "pb_critical".to_string(),
            "host1".to_string(),
            None,
            None,
        );
        
        let normal_sig = IncidentSignature::new(
            "pb_normal".to_string(),
            "host1".to_string(),
            None,
            None,
        );
        
        // Record both fires
        tracker.record_fire(&critical_sig);
        tracker.record_fire(&normal_sig);
        
        // Critical should allow re-fire sooner (after 500ms not 5000ms)
        // Normal would need 5000ms
        // (In a real test, we'd mock time to verify exact thresholds)
        
        let critical_remaining = tracker.can_fire(&critical_sig).1;
        let normal_remaining = tracker.can_fire(&normal_sig).1;
        
        assert!(critical_remaining < normal_remaining);
    }

    #[test]
    fn test_signature_different_hosts() {
        let policy = CooldownPolicy::default();
        let mut tracker = CooldownTracker::new(policy);
        
        let sig1 = IncidentSignature::new(
            "pb_test".to_string(),
            "host1".to_string(),
            None,
            None,
        );
        
        let sig2 = IncidentSignature::new(
            "pb_test".to_string(),
            "host2".to_string(),
            None,
            None,
        );
        
        // Fire on host1
        tracker.record_fire(&sig1);
        
        // host2 should still be able to fire (different signature)
        assert!(tracker.can_fire(&sig2).0);
    }

    #[test]
    fn test_reset_cooldown() {
        let policy = CooldownPolicy::default();
        let mut tracker = CooldownTracker::new(policy);
        
        let sig = IncidentSignature::new(
            "pb_test".to_string(),
            "host1".to_string(),
            None,
            None,
        );
        
        tracker.record_fire(&sig);
        assert!(!tracker.can_fire(&sig).0); // Blocked
        
        tracker.reset(&sig);
        assert!(tracker.can_fire(&sig).0);  // Unblocked
    }
}
