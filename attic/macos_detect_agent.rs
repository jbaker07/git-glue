/// macos_detect_agent: Bridges macos_detect library with EDR pipeline
/// Reads segments from telemetry_output/segments/index.json
/// Converts to facts, runs playbooks, persists to analysis.db
/// Emits ExplanationBundles for API consumption

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

fn main() {
    eprintln!("macos_detect_agent: Integration stub");
    eprintln!("This agent bridges macos_detect to the EDR pipeline");
    eprintln!("In production: reads segments → runs playbooks → stores incidents+explanations");
}

#[derive(Debug, Serialize, Deserialize)]
struct SegmentIndex {
    segments: Vec<SegmentRef>,
    last_updated_ts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SegmentRef {
    segment_id: String,
    path: String,
    ts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SegmentData {
    host: String,
    segment_id: String,
    ts: u64,
    events: Vec<RawEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawEvent {
    event_type: String,
    data: serde_json::Value,
}
