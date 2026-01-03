//! Production-ready capabilities manifest and coverage registry for adaptive incident compilation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Capabilities Manifest
// ============================================================================

/// Declares what each data source can provide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesManifest {
    /// List of source names (e.g., "CrowdStrike", "SentinelOne")
    pub sources: Vec<String>,
    /// Source capabilities
    pub capabilities: HashMap<String, SourceCapabilities>,
    /// Global settings
    #[serde(default)]
    pub settings: ManifestSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCapabilities {
    /// Event types this source provides (e.g., ["ProcessCreation", "NetworkConnection"])
    pub event_types: Vec<String>,
    /// Primitives this source covers (e.g., ["process_start", "dns_query"])
    pub primitives: Vec<String>,
    /// Data fidelity level
    pub fidelity: Fidelity,
    /// Trust level for this source
    pub trust_level: TrustLevel,
    /// Data timeliness (e.g., "real-time", "5-minute-delay", "daily-batch")
    pub timeliness: String,
    /// Retention period (e.g., "30d", "90d", "1y")
    pub retention: Option<String>,
    /// Known limitations or gaps
    pub limitations: Vec<String>,
    /// Weight for correlation (higher = more trusted)
    #[serde(default = "default_weight")]
    pub weight: u8,
}

fn default_weight() -> u8 {
    50
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fidelity {
    /// High-fidelity, low noise
    High,
    /// Medium fidelity, some noise
    Medium,
    /// Low fidelity, high noise
    Low,
    /// Raw telemetry, unprocessed
    Raw,
}

impl Default for Fidelity {
    fn default() -> Self {
        Fidelity::Medium
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Cryptographically signed/verified
    Signed,
    /// Authenticated but not signed
    Authenticated,
    /// Best-effort, some validation
    BestEffort,
    /// Unverified/raw
    Unverified,
}

impl Default for TrustLevel {
    fn default() -> Self {
        TrustLevel::Authenticated
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestSettings {
    /// Minimum sources required for high confidence
    #[serde(default = "default_min_sources")]
    pub min_sources_for_high_confidence: usize,
    /// Default confidence when only one source
    #[serde(default = "default_single_source_confidence")]
    pub single_source_confidence: f32,
    /// Enable automatic gap detection
    #[serde(default)]
    pub auto_gap_detection: bool,
}

fn default_min_sources() -> usize {
    2
}
fn default_single_source_confidence() -> f32 {
    0.6
}

impl CapabilitiesManifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            capabilities: HashMap::new(),
            settings: ManifestSettings::default(),
        }
    }

    /// Load manifest from YAML or JSON file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest file: {}", path))?;

        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let manifest: Self = match ext {
            "yaml" | "yml" => {
                serde_yaml::from_str(&data).with_context(|| "Failed to parse YAML manifest")?
            }
            "json" => {
                serde_json::from_str(&data).with_context(|| "Failed to parse JSON manifest")?
            }
            _ => anyhow::bail!(
                "Unsupported manifest format: {}. Use .yaml, .yml, or .json",
                ext
            ),
        };

        manifest.validate()?;
        Ok(manifest)
    }

    /// Load from string (for embedded configs)
    pub fn load_from_str(content: &str, format: &str) -> Result<Self> {
        let manifest: Self = match format {
            "yaml" | "yml" => serde_yaml::from_str(content)?,
            "json" => serde_json::from_str(content)?,
            _ => anyhow::bail!("Unsupported format: {}", format),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<()> {
        if self.sources.is_empty() {
            anyhow::bail!("Manifest must declare at least one source");
        }

        // Check that all sources have capabilities defined
        for source in &self.sources {
            if !self.capabilities.contains_key(source) {
                anyhow::bail!("Source '{}' declared but no capabilities defined", source);
            }
        }

        Ok(())
    }

    /// Add a source with its capabilities
    pub fn add_source(&mut self, name: &str, caps: SourceCapabilities) {
        if !self.sources.contains(&name.to_string()) {
            self.sources.push(name.to_string());
        }
        self.capabilities.insert(name.to_string(), caps);
    }

    /// Get capabilities for a specific source
    pub fn get_capabilities(&self, source: &str) -> Option<&SourceCapabilities> {
        self.capabilities.get(source)
    }

    /// Get all sources that provide a specific primitive
    pub fn sources_for_primitive(&self, primitive: &str) -> Vec<String> {
        self.capabilities
            .iter()
            .filter(|(_, caps)| caps.primitives.contains(&primitive.to_string()))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get all primitives covered by any source
    pub fn all_primitives(&self) -> HashSet<String> {
        self.capabilities
            .values()
            .flat_map(|caps| caps.primitives.iter().cloned())
            .collect()
    }

    /// Get all event types covered by any source
    pub fn all_event_types(&self) -> HashSet<String> {
        self.capabilities
            .values()
            .flat_map(|caps| caps.event_types.iter().cloned())
            .collect()
    }

    /// Calculate expected coverage for a set of active sources
    pub fn calculate_coverage(&self, active_sources: &[String]) -> CoverageReport {
        let all_primitives = self.all_primitives();
        let mut covered: HashSet<String> = HashSet::new();
        let mut source_coverage: HashMap<String, Vec<String>> = HashMap::new();

        for source in active_sources {
            if let Some(caps) = self.capabilities.get(source) {
                for prim in &caps.primitives {
                    covered.insert(prim.clone());
                    source_coverage
                        .entry(prim.clone())
                        .or_default()
                        .push(source.clone());
                }
            }
        }

        let missing: HashSet<String> = all_primitives.difference(&covered).cloned().collect();
        let coverage_percent = if all_primitives.is_empty() {
            100.0
        } else {
            (covered.len() as f32 / all_primitives.len() as f32) * 100.0
        };

        CoverageReport {
            covered_primitives: covered,
            missing_primitives: missing,
            coverage_percentage: coverage_percent,
            source_coverage,
        }
    }
}

impl Default for CapabilitiesManifest {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Coverage Report
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub covered_primitives: HashSet<String>,
    pub missing_primitives: HashSet<String>,
    pub coverage_percentage: f32,
    pub source_coverage: HashMap<String, Vec<String>>,
}

// ============================================================================
// Coverage Registry
// ============================================================================

/// Runtime registry tracking actual coverage from ingested data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageRegistry {
    /// Primitives observed (primitive -> sources that provided it)
    pub observed_primitives: HashMap<String, Vec<String>>,
    /// Source weights for correlation
    pub source_weights: HashMap<String, u8>,
    /// Proof thresholds per primitive (how many sources needed for high confidence)
    pub proof_thresholds: HashMap<String, u8>,
    /// Last update time per source
    pub last_seen: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

impl CoverageRegistry {
    pub fn new() -> Self {
        Self {
            observed_primitives: HashMap::new(),
            source_weights: HashMap::new(),
            proof_thresholds: HashMap::new(),
            last_seen: HashMap::new(),
        }
    }

    /// Record observation of a primitive from a source
    pub fn record_observation(&mut self, primitive: &str, source: &str) {
        self.observed_primitives
            .entry(primitive.to_string())
            .or_default()
            .push(source.to_string());
        self.last_seen
            .insert(source.to_string(), chrono::Utc::now());
    }

    /// Set weight for a source
    pub fn set_source_weight(&mut self, source: &str, weight: u8) {
        self.source_weights.insert(source.to_string(), weight);
    }

    /// Set proof threshold for a primitive
    pub fn set_proof_threshold(&mut self, primitive: &str, threshold: u8) {
        self.proof_thresholds
            .insert(primitive.to_string(), threshold);
    }

    /// Check if a primitive has sufficient proof
    pub fn has_sufficient_proof(&self, primitive: &str) -> bool {
        let threshold = self.proof_thresholds.get(primitive).copied().unwrap_or(1);
        let sources = self
            .observed_primitives
            .get(primitive)
            .map(|s| s.len())
            .unwrap_or(0);
        sources >= threshold as usize
    }

    /// Calculate confidence for a primitive based on source weights
    pub fn calculate_confidence(&self, primitive: &str) -> f32 {
        let sources = match self.observed_primitives.get(primitive) {
            Some(s) => s,
            None => return 0.0,
        };

        if sources.is_empty() {
            return 0.0;
        }

        let total_weight: u32 = sources
            .iter()
            .map(|s| self.source_weights.get(s).copied().unwrap_or(50) as u32)
            .sum();

        // Normalize to 0-1 range (max weight per source is 100, max sources typically ~5)
        (total_weight as f32 / 500.0).min(1.0)
    }

    /// Get all observed primitives
    pub fn observed(&self) -> Vec<String> {
        self.observed_primitives.keys().cloned().collect()
    }

    /// Get sources that observed a primitive
    pub fn sources_for(&self, primitive: &str) -> Vec<String> {
        self.observed_primitives
            .get(primitive)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for CoverageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Preset Manifests
// ============================================================================

impl CapabilitiesManifest {
    /// Create a preset manifest for common EDR stack
    pub fn preset_enterprise_edr() -> Self {
        let mut manifest = Self::new();

        manifest.add_source(
            "CrowdStrike",
            SourceCapabilities {
                event_types: vec![
                    "Detection",
                    "ProcessRollup",
                    "NetworkConnect",
                    "FileWrite",
                    "DnsRequest",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                primitives: vec![
                    "process_creation",
                    "network_connection",
                    "file_write",
                    "dns_query",
                    "process_termination",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                fidelity: Fidelity::High,
                trust_level: TrustLevel::Signed,
                timeliness: "real-time".to_string(),
                retention: Some("90d".to_string()),
                limitations: vec!["Limited Linux kernel visibility".to_string()],
                weight: 80,
            },
        );

        manifest.add_source(
            "SentinelOne",
            SourceCapabilities {
                event_types: vec![
                    "Threat",
                    "ProcessCreation",
                    "NetworkConnection",
                    "FileModification",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                primitives: vec![
                    "process_creation",
                    "network_connection",
                    "file_modification",
                    "registry_modification",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                fidelity: Fidelity::High,
                trust_level: TrustLevel::Authenticated,
                timeliness: "real-time".to_string(),
                retention: Some("30d".to_string()),
                limitations: vec![],
                weight: 75,
            },
        );

        manifest.add_source(
            "Defender",
            SourceCapabilities {
                event_types: vec![
                    "Alert",
                    "DeviceProcessEvents",
                    "DeviceNetworkEvents",
                    "DeviceFileEvents",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                primitives: vec![
                    "process_creation",
                    "network_connection",
                    "file_access",
                    "registry_access",
                    "logon_event",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                fidelity: Fidelity::Medium,
                trust_level: TrustLevel::Authenticated,
                timeliness: "near-real-time".to_string(),
                retention: Some("30d".to_string()),
                limitations: vec!["Windows-only".to_string()],
                weight: 70,
            },
        );

        manifest.settings = ManifestSettings {
            min_sources_for_high_confidence: 2,
            single_source_confidence: 0.65,
            auto_gap_detection: true,
        };

        manifest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = CapabilitiesManifest::preset_enterprise_edr();
        assert!(manifest.sources.len() >= 3);
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_coverage_calculation() {
        let manifest = CapabilitiesManifest::preset_enterprise_edr();
        let report =
            manifest.calculate_coverage(&["CrowdStrike".to_string(), "SentinelOne".to_string()]);
        assert!(report.coverage_percentage > 0.0);
    }

    #[test]
    fn test_registry_confidence() {
        let mut registry = CoverageRegistry::new();
        registry.set_source_weight("CrowdStrike", 80);
        registry.set_source_weight("SentinelOne", 75);

        registry.record_observation("process_creation", "CrowdStrike");
        registry.record_observation("process_creation", "SentinelOne");

        let confidence = registry.calculate_confidence("process_creation");
        assert!(confidence > 0.2);
    }
}
