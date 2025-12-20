/// policy_loader.rs: Policy enforcement + corroboration system (P1-1)
///
/// Loads retention policies, applies them to capture/edr_locald,
/// and implements signal corroboration (multiple incident signals = higher confidence)

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::retention_contract::RetentionPolicy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Retention policy settings
    pub retention: RetentionPolicy,
    
    /// Corroboration thresholds: how many signals needed for confidence levels
    pub corroboration: CorroborationPolicy,
    
    /// Policy version for tracking updates
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorroborationPolicy {
    /// Minimum signals for "low" confidence incident
    pub low_confidence_signals: u32,
    
    /// Minimum signals for "medium" confidence incident
    pub medium_confidence_signals: u32,
    
    /// Minimum signals for "high" confidence incident
    pub high_confidence_signals: u32,
    
    /// Signal decay window: signals older than this (ms) don't count
    pub signal_decay_window_ms: u64,
    
    /// Enable corroboration (if disabled, single signal fires incidents)
    pub enabled: bool,
}

impl Default for CorroborationPolicy {
    fn default() -> Self {
        CorroborationPolicy {
            low_confidence_signals: 1,      // 1 signal = low confidence
            medium_confidence_signals: 2,   // 2 signals = medium confidence
            high_confidence_signals: 3,     // 3+ signals = high confidence
            signal_decay_window_ms: 3600000, // 1 hour
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_id: String,
    pub playbook_id: String,
    pub signal_type: String,    // "behavior", "ioc", "anomaly", "baseline_violation"
    pub confidence: f32,        // 0.0-1.0
    pub ts_ms: u64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CorroborationResult {
    pub overall_confidence: f32,
    pub signal_count: u32,
    pub grouped_signals: Vec<Vec<Signal>>,
    pub recommended_severity: String,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        PolicyConfig {
            retention: RetentionPolicy::default(),
            corroboration: CorroborationPolicy::default(),
            version: "1.0".to_string(),
        }
    }
}

impl PolicyConfig {
    /// Load policy from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(PolicyConfig::default());
        }
        
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read policy file: {}", e))?;
        
        serde_json::from_str::<PolicyConfig>(&contents)
            .map_err(|e| format!("Failed to parse policy file: {}", e))
    }
    
    /// Save policy to JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize policy: {}", e))?;
        fs::write(path, json_str)
            .map_err(|e| format!("Failed to write policy file: {}", e))
    }
}

/// Signal aggregator: groups signals by playbook and scores overall confidence
pub struct SignalAggregator {
    signals: Vec<Signal>,
    policy: CorroborationPolicy,
}

impl SignalAggregator {
    pub fn new(policy: CorroborationPolicy) -> Self {
        SignalAggregator {
            signals: Vec::new(),
            policy,
        }
    }
    
    /// Add a signal for corroboration
    pub fn add_signal(&mut self, signal: Signal) {
        self.signals.push(signal);
    }
    
    /// Filter signals within decay window
    fn active_signals(&self, now_ms: u64) -> Vec<Signal> {
        self.signals
            .iter()
            .filter(|s| (now_ms.saturating_sub(s.ts_ms)) < self.policy.signal_decay_window_ms)
            .cloned()
            .collect()
    }
    
    /// Group signals by playbook_id
    fn group_by_playbook(signals: &[Signal]) -> Vec<Vec<Signal>> {
        let mut groups: std::collections::HashMap<String, Vec<Signal>> = std::collections::HashMap::new();
        
        for signal in signals {
            groups
                .entry(signal.playbook_id.clone())
                .or_insert_with(Vec::new)
                .push(signal.clone());
        }
        
        let mut result: Vec<_> = groups.into_values().collect();
        result.sort_by(|a, b| b.len().cmp(&a.len())); // Largest groups first
        result
    }
    
    /// Compute overall confidence from signal ensemble
    fn compute_confidence(signals: &[Signal]) -> f32 {
        if signals.is_empty() {
            return 0.0;
        }
        
        // Average confidence of signals, weighted by count
        let avg_conf: f32 = signals.iter().map(|s| s.confidence).sum::<f32>() / signals.len() as f32;
        
        // Bonus for multiple independent signals
        let count_factor = (signals.len() as f32).ln() / 2.0; // log boost
        
        let combined = avg_conf + (count_factor * 0.2); // 20% bonus per additional signal
        combined.min(1.0) // Clamp to [0, 1]
    }
    
    /// Evaluate corroboration and recommend severity
    pub fn evaluate(&self, now_ms: u64) -> CorroborationResult {
        if !self.policy.enabled {
            // Corroboration disabled: single signal fires
            let active = self.active_signals(now_ms);
            if active.is_empty() {
                return CorroborationResult {
                    overall_confidence: 0.0,
                    signal_count: 0,
                    grouped_signals: Vec::new(),
                    recommended_severity: "low".to_string(),
                };
            }
            
            let conf = Self::compute_confidence(&active);
            return CorroborationResult {
                overall_confidence: conf,
                signal_count: active.len() as u32,
                grouped_signals: Self::group_by_playbook(&active),
                recommended_severity: severity_from_confidence(conf),
            };
        }
        
        let active = self.active_signals(now_ms);
        let count = active.len() as u32;
        let confidence = Self::compute_confidence(&active);
        let grouped = Self::group_by_playbook(&active);
        
        // Determine severity based on signal count
        let severity = if count >= self.policy.high_confidence_signals {
            "critical".to_string()
        } else if count >= self.policy.medium_confidence_signals {
            "high".to_string()
        } else if count >= self.policy.low_confidence_signals {
            "medium".to_string()
        } else {
            "low".to_string()
        };
        
        CorroborationResult {
            overall_confidence: confidence,
            signal_count: count,
            grouped_signals: grouped,
            recommended_severity: severity,
        }
    }
    
    /// Clear all signals (after incident fired or timeout)
    pub fn clear(&mut self) {
        self.signals.clear();
    }
}

fn severity_from_confidence(conf: f32) -> String {
    if conf >= 0.9 {
        "critical".to_string()
    } else if conf >= 0.7 {
        "high".to_string()
    } else if conf >= 0.5 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_load_default() {
        let policy = PolicyConfig::default();
        assert_eq!(policy.version, "1.0");
        assert!(policy.corroboration.enabled);
    }

    #[test]
    fn test_signal_aggregation() {
        let mut agg = SignalAggregator::new(CorroborationPolicy::default());
        let now = 1000000u64;
        
        agg.add_signal(Signal {
            signal_id: "sig_1".to_string(),
            playbook_id: "pb_1".to_string(),
            signal_type: "behavior".to_string(),
            confidence: 0.8,
            ts_ms: now,
            description: "Signal 1".to_string(),
        });
        
        agg.add_signal(Signal {
            signal_id: "sig_2".to_string(),
            playbook_id: "pb_1".to_string(),
            signal_type: "ioc".to_string(),
            confidence: 0.9,
            ts_ms: now,
            description: "Signal 2".to_string(),
        });
        
        let result = agg.evaluate(now);
        assert_eq!(result.signal_count, 2);
        assert!(result.overall_confidence > 0.8);
        assert_eq!(result.recommended_severity, "high");
    }

    #[test]
    fn test_signal_decay() {
        let policy = CorroborationPolicy {
            signal_decay_window_ms: 1000,
            ..Default::default()
        };
        let mut agg = SignalAggregator::new(policy);
        let now = 1000000u64;
        
        agg.add_signal(Signal {
            signal_id: "sig_old".to_string(),
            playbook_id: "pb_1".to_string(),
            signal_type: "behavior".to_string(),
            confidence: 0.8,
            ts_ms: now - 2000,  // Beyond decay window
            description: "Old signal".to_string(),
        });
        
        let result = agg.evaluate(now);
        assert_eq!(result.signal_count, 0);
    }

    #[test]
    fn test_corroboration_disabled() {
        let mut policy = CorroborationPolicy::default();
        policy.enabled = false;
        
        let mut agg = SignalAggregator::new(policy);
        let now = 1000000u64;
        
        agg.add_signal(Signal {
            signal_id: "sig_1".to_string(),
            playbook_id: "pb_1".to_string(),
            signal_type: "behavior".to_string(),
            confidence: 0.6,
            ts_ms: now,
            description: "Single low-confidence signal".to_string(),
        });
        
        let result = agg.evaluate(now);
        assert_eq!(result.signal_count, 1);
        // Even low conf signal fires when corroboration disabled
        assert!(result.overall_confidence > 0.0);
    }
}
