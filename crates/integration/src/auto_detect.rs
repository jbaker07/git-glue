//! Auto-detect capabilities from live event data.

use crate::adapter::NormalizedEvent;
use crate::capabilities::{CapabilitiesManifest, Fidelity, SourceCapabilities, TrustLevel};
use std::collections::{HashMap, HashSet};

/// Scan events and infer available sources, fidelity, and trust
pub fn auto_detect_capabilities(events: &[NormalizedEvent]) -> CapabilitiesManifest {
    let mut manifest = CapabilitiesManifest::new();

    // Group events by source
    let mut by_source: HashMap<String, Vec<&NormalizedEvent>> = HashMap::new();
    for event in events {
        by_source
            .entry(event.source.clone())
            .or_default()
            .push(event);
    }

    // Analyze each source
    for (source, source_events) in by_source {
        let event_types: HashSet<String> =
            source_events.iter().map(|e| e.event_type.clone()).collect();

        let primitives = infer_primitives(&event_types);
        let fidelity = infer_fidelity(&source_events);
        let trust_level = infer_trust_level(&source);
        let timeliness = infer_timeliness(&source_events);

        let caps = SourceCapabilities {
            event_types: event_types.into_iter().collect(),
            primitives,
            fidelity,
            trust_level,
            timeliness,
            retention: None,
            limitations: Vec::new(),
            weight: infer_weight(&source),
        };

        manifest.add_source(&source, caps);
    }

    manifest
}

fn infer_primitives(event_types: &HashSet<String>) -> Vec<String> {
    let mut primitives = Vec::new();

    for et in event_types {
        let lower = et.to_lowercase();
        if lower.contains("process") {
            primitives.push("process_creation".to_string());
        }
        if lower.contains("network") || lower.contains("connect") {
            primitives.push("network_connection".to_string());
        }
        if lower.contains("file") {
            primitives.push("file_modification".to_string());
        }
        if lower.contains("dns") {
            primitives.push("dns_query".to_string());
        }
        if lower.contains("registry") {
            primitives.push("registry_modification".to_string());
        }
        if lower.contains("auth") || lower.contains("logon") {
            primitives.push("authentication".to_string());
        }
    }

    // Dedup
    primitives.sort();
    primitives.dedup();
    primitives
}

fn infer_fidelity(events: &[&NormalizedEvent]) -> Fidelity {
    // Check for detailed process info
    let has_cmdline = events.iter().any(|e| {
        e.process
            .as_ref()
            .map(|p| p.command_line.is_some())
            .unwrap_or(false)
    });

    let has_network_detail = events.iter().any(|e| {
        e.network
            .as_ref()
            .map(|n| n.dest_ip.is_some() && n.dest_port.is_some())
            .unwrap_or(false)
    });

    if has_cmdline && has_network_detail {
        Fidelity::High
    } else if has_cmdline || has_network_detail {
        Fidelity::Medium
    } else {
        Fidelity::Low
    }
}

fn infer_trust_level(source: &str) -> TrustLevel {
    let known_signed = ["CrowdStrike", "Defender", "Carbon Black"];
    let known_authenticated = ["SentinelOne", "Cortex XDR", "Sophos"];

    if known_signed.iter().any(|s| source.contains(s)) {
        TrustLevel::Signed
    } else if known_authenticated.iter().any(|s| source.contains(s)) {
        TrustLevel::Authenticated
    } else {
        TrustLevel::BestEffort
    }
}

fn infer_timeliness(events: &[&NormalizedEvent]) -> String {
    if events.is_empty() {
        return "unknown".to_string();
    }

    // Check time spread
    let mut timestamps: Vec<_> = events.iter().map(|e| e.timestamp).collect();
    timestamps.sort();

    if let (Some(first), Some(last)) = (timestamps.first(), timestamps.last()) {
        let duration = (*last - *first).num_seconds();
        if duration < 60 {
            "real-time".to_string()
        } else if duration < 300 {
            "near-real-time".to_string()
        } else {
            "batch".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

fn infer_weight(source: &str) -> u8 {
    match source {
        s if s.contains("CrowdStrike") => 85,
        s if s.contains("SentinelOne") => 80,
        s if s.contains("Defender") => 75,
        s if s.contains("Carbon Black") => 75,
        s if s.contains("Cortex") => 75,
        _ => 50,
    }
}

/// Legacy function for backward compatibility
pub fn auto_detect_capabilities_json(events: &[serde_json::Value]) -> CapabilitiesManifest {
    // Convert JSON events to normalized events (best effort)
    let normalized: Vec<NormalizedEvent> = events
        .iter()
        .filter_map(|v| {
            let source = v.get("source")?.as_str()?.to_string();
            let event_type = v
                .get("event_type")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown")
                .to_string();
            let host = v
                .get("host")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown")
                .to_string();

            Some(NormalizedEvent {
                id: uuid::Uuid::new_v4().to_string(),
                source,
                event_type,
                timestamp: chrono::Utc::now(),
                host,
                user: v.get("user").and_then(|u| u.as_str()).map(String::from),
                severity: v.get("severity").and_then(|s| s.as_str()).map(String::from),
                description: None,
                process: None,
                network: None,
                file: None,
                raw: v.clone(),
                custom_fields: HashMap::new(),
            })
        })
        .collect();

    auto_detect_capabilities(&normalized)
}
