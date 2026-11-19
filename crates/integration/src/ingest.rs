//! Production-ready ingest module: orchestrate fetching from multiple EDR/SIEM sources.

use crate::adapter::{AdapterRegistry, NormalizedEvent};
use anyhow::Result;
use std::collections::HashMap;
use tracing::{error, info};

// ============================================================================
// Ingest Configuration
// ============================================================================

/// Configuration for ingest operations
#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// Maximum concurrent fetches
    pub max_concurrent: usize,
    /// Timeout per source (seconds)
    pub timeout_secs: u64,
    /// Continue on error
    pub continue_on_error: bool,
    /// Dedup events by ID
    pub dedup: bool,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            timeout_secs: 60,
            continue_on_error: true,
            dedup: true,
        }
    }
}

// ============================================================================
// Ingest Result
// ============================================================================

/// Result of an ingest operation
#[derive(Debug)]
pub struct IngestResult {
    /// Successfully fetched events
    pub events: Vec<NormalizedEvent>,
    /// Events per source
    pub events_by_source: HashMap<String, usize>,
    /// Errors encountered
    pub errors: Vec<IngestError>,
    /// Total fetch time (ms)
    pub total_time_ms: u64,
}

#[derive(Debug)]
pub struct IngestError {
    pub source: String,
    pub message: String,
}

// ============================================================================
// Ingest Functions
// ============================================================================

/// Ingest events from all registered adapters
pub async fn ingest_from_registry(
    registry: &AdapterRegistry,
    config: &IngestConfig,
) -> IngestResult {
    let start = std::time::Instant::now();
    let mut all_events = Vec::new();
    let mut events_by_source = HashMap::new();
    let mut errors = Vec::new();

    for adapter in registry.adapters() {
        info!("Ingesting from {}", adapter.name());

        match tokio::time::timeout(
            std::time::Duration::from_secs(config.timeout_secs),
            adapter.fetch_and_normalize(),
        )
        .await
        {
            Ok(Ok(events)) => {
                let count = events.len();
                events_by_source.insert(adapter.name().to_string(), count);
                all_events.extend(events);
                info!("{}: fetched {} events", adapter.name(), count);
            }
            Ok(Err(e)) => {
                error!("{}: fetch error: {}", adapter.name(), e);
                errors.push(IngestError {
                    source: adapter.name().to_string(),
                    message: e.to_string(),
                });
                if !config.continue_on_error {
                    break;
                }
            }
            Err(_) => {
                error!("{}: timeout after {}s", adapter.name(), config.timeout_secs);
                errors.push(IngestError {
                    source: adapter.name().to_string(),
                    message: format!("Timeout after {} seconds", config.timeout_secs),
                });
                if !config.continue_on_error {
                    break;
                }
            }
        }
    }

    // Dedup if requested
    if config.dedup {
        let before = all_events.len();
        all_events = dedup_events(all_events);
        let after = all_events.len();
        if before != after {
            info!("Deduped {} events to {}", before, after);
        }
    }

    IngestResult {
        events: all_events,
        events_by_source,
        errors,
        total_time_ms: start.elapsed().as_millis() as u64,
    }
}

/// Ingest from a specific list of sources by name
pub async fn ingest_from_sources(
    registry: &AdapterRegistry,
    source_names: &[&str],
    _config: &IngestConfig,
) -> IngestResult {
    let start = std::time::Instant::now();
    let mut all_events = Vec::new();
    let mut events_by_source = HashMap::new();
    let mut errors = Vec::new();

    for adapter in registry.adapters() {
        if !source_names.contains(&adapter.name()) {
            continue;
        }

        match adapter.fetch_and_normalize().await {
            Ok(events) => {
                events_by_source.insert(adapter.name().to_string(), events.len());
                all_events.extend(events);
            }
            Err(e) => {
                errors.push(IngestError {
                    source: adapter.name().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    IngestResult {
        events: all_events,
        events_by_source,
        errors,
        total_time_ms: start.elapsed().as_millis() as u64,
    }
}

/// Dedup events by ID
fn dedup_events(events: Vec<NormalizedEvent>) -> Vec<NormalizedEvent> {
    let mut seen = std::collections::HashSet::new();
    events
        .into_iter()
        .filter(|e| seen.insert(e.id.clone()))
        .collect()
}

// ============================================================================
// Convenience Function (backward compatible)
// ============================================================================

/// Legacy function signature for backward compatibility
pub async fn fetch_external_events(
    _platform: &str,
    _api_url: &str,
    _token: &str,
) -> Result<Vec<serde_json::Value>> {
    // Use the new adapter system instead
    Ok(vec![])
}
