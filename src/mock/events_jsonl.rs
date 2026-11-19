/// JSONL reader for core::Event fixtures
/// Tolerant parsing: skip malformed lines, continue reading
use crate::core::Event;
use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Load core::Event JSONL fixture file
/// Skips unparseable lines, returns count of successful reads
pub fn load_events_jsonl<P: AsRef<Path>>(path: P) -> Result<(Vec<Event>, usize)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut events = Vec::new();
    let mut skipped = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<Event>(trimmed) {
            Ok(evt) => events.push(evt),
            Err(_) => {
                skipped += 1;
            }
        }
    }

    Ok((events, skipped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty() {
        // Empty file returns 0 events
        let result = load_events_jsonl("/dev/null");
        assert!(result.is_ok());
        let (events, _) = result.unwrap();
        assert!(events.is_empty());
    }
}
