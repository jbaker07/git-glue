//! Stub trait implementations for sensors
//! TODO: Refactor each sensor to emit core::Event instead of using telemetry/gnn/trust hooks

use crate::core::Event;
use anyhow::Result;

/// Provides a default empty impl for sensors that haven't been refactored yet
pub fn stub_poll(_host: &str) -> Result<Vec<Event>> {
    Ok(Vec::new())
}
