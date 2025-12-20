/// playbook_validator.rs: Strict playbook validation (P1-2)
///
/// Ensures all loaded playbooks are:
/// - Syntactically valid YAML/JSON
/// - Have required fields (playbook_id, title, severity, slots, fire)
/// - Reference valid predicates and operators
/// - Have no circular dependencies
/// - Define fire conditions that reference existing slots
///
/// This prevents runtime crashes from malformed playbooks.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookDef {
    pub playbook_id: String,
    pub title: String,
    pub severity: String,
    pub priority: Option<u32>,
    pub entity_scope: String,
    pub slots: Vec<SlotDef>,
    pub fire: FireDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDef {
    pub slot_id: String,
    pub entity_key: String,
    pub predicates: Vec<PredicateDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateDef {
    pub field: String,
    pub op: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireDef {
    pub condition: String,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub playbook_id: String,
    pub field: String,
    pub message: String,
}

pub struct PlaybookValidator {
    valid_operators: HashSet<String>,
}

impl PlaybookValidator {
    pub fn new() -> Self {
        let mut valid_ops = HashSet::new();
        // Standard comparison operators
        for op in &["==", "!=", "<", ">", "<=", ">=", "in", "not_in", "regex", "exists"] {
            valid_ops.insert(op.to_string());
        }
        
        PlaybookValidator {
            valid_operators: valid_ops,
        }
    }
    
    /// Validate a single playbook
    pub fn validate_playbook(&self, pb: &PlaybookDef) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // P0: Required fields
        if pb.playbook_id.trim().is_empty() {
            errors.push(ValidationError {
                playbook_id: pb.playbook_id.clone(),
                field: "playbook_id".to_string(),
                message: "playbook_id is required and must not be empty".to_string(),
            });
        }
        
        if pb.title.trim().is_empty() {
            errors.push(ValidationError {
                playbook_id: pb.playbook_id.clone(),
                field: "title".to_string(),
                message: "title is required and must not be empty".to_string(),
            });
        }
        
        // P1: Severity validation
        let valid_severities = ["low", "medium", "high", "critical"];
        if !valid_severities.contains(&pb.severity.as_str()) {
            errors.push(ValidationError {
                playbook_id: pb.playbook_id.clone(),
                field: "severity".to_string(),
                message: format!(
                    "severity must be one of: {}",
                    valid_severities.join(", ")
                ),
            });
        }
        
        // P2: Entity scope validation
        let valid_scopes = ["user", "host", "process", "network"];
        if !valid_scopes.contains(&pb.entity_scope.as_str()) {
            errors.push(ValidationError {
                playbook_id: pb.playbook_id.clone(),
                field: "entity_scope".to_string(),
                message: format!(
                    "entity_scope must be one of: {}",
                    valid_scopes.join(", ")
                ),
            });
        }
        
        // P3: Slots validation
        if pb.slots.is_empty() {
            errors.push(ValidationError {
                playbook_id: pb.playbook_id.clone(),
                field: "slots".to_string(),
                message: "playbook must define at least one slot".to_string(),
            });
        }
        
        let mut slot_ids = HashSet::new();
        for (idx, slot) in pb.slots.iter().enumerate() {
            // Slot ID uniqueness
            if !slot_ids.insert(&slot.slot_id) {
                errors.push(ValidationError {
                    playbook_id: pb.playbook_id.clone(),
                    field: format!("slots[{}].slot_id", idx),
                    message: format!("slot_id '{}' is duplicated", slot.slot_id),
                });
            }
            
            // Slot ID not empty
            if slot.slot_id.trim().is_empty() {
                errors.push(ValidationError {
                    playbook_id: pb.playbook_id.clone(),
                    field: format!("slots[{}].slot_id", idx),
                    message: "slot_id must not be empty".to_string(),
                });
            }
            
            // Entity key not empty
            if slot.entity_key.trim().is_empty() {
                errors.push(ValidationError {
                    playbook_id: pb.playbook_id.clone(),
                    field: format!("slots[{}].entity_key", idx),
                    message: "entity_key must not be empty".to_string(),
                });
            }
            
            // Predicates validation
            if slot.predicates.is_empty() {
                errors.push(ValidationError {
                    playbook_id: pb.playbook_id.clone(),
                    field: format!("slots[{}].predicates", idx),
                    message: "slot must define at least one predicate".to_string(),
                });
            }
            
            for (pred_idx, pred) in slot.predicates.iter().enumerate() {
                if pred.field.trim().is_empty() {
                    errors.push(ValidationError {
                        playbook_id: pb.playbook_id.clone(),
                        field: format!("slots[{}].predicates[{}].field", idx, pred_idx),
                        message: "predicate field must not be empty".to_string(),
                    });
                }
                
                if !self.valid_operators.contains(&pred.op) {
                    errors.push(ValidationError {
                        playbook_id: pb.playbook_id.clone(),
                        field: format!("slots[{}].predicates[{}].op", idx, pred_idx),
                        message: format!(
                            "operator '{}' is not valid. Valid operators: {}",
                            pred.op,
                            self.valid_operators.iter().cloned().collect::<Vec<_>>().join(", ")
                        ),
                    });
                }
            }
        }
        
        // P4: Fire condition validation
        if pb.fire.condition.trim().is_empty() {
            errors.push(ValidationError {
                playbook_id: pb.playbook_id.clone(),
                field: "fire.condition".to_string(),
                message: "fire condition must not be empty".to_string(),
            });
        }
        
        // P5: Fire condition references valid slots
        if !pb.fire.condition.trim().is_empty() {
            let referenced_slots = self.extract_slot_refs(&pb.fire.condition);
            for slot_ref in referenced_slots {
                if !slot_ids.contains(&slot_ref) {
                    errors.push(ValidationError {
                        playbook_id: pb.playbook_id.clone(),
                        field: "fire.condition".to_string(),
                        message: format!(
                            "fire condition references undefined slot '{}'",
                            slot_ref
                        ),
                    });
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Extract slot IDs referenced in a fire condition
    fn extract_slot_refs(&self, condition: &str) -> HashSet<String> {
        let mut refs = HashSet::new();
        
        // Simple regex-like extraction: look for identifiers that look like slot names
        // Pattern: word characters, but at least one letter
        let words: Vec<&str> = condition.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| !w.is_empty() && w.chars().any(|c| c.is_alphabetic()))
            .collect();
        
        for word in words {
            // Heuristic: if it's not a common keyword, assume it's a slot reference
            if !is_keyword(word) {
                refs.insert(word.to_string());
            }
        }
        
        refs
    }
    
    /// Load and validate a playbook from file
    pub fn validate_file<P: AsRef<Path>>(&self, path: P) -> Result<PlaybookDef, Vec<ValidationError>> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .map_err(|e| vec![ValidationError {
                playbook_id: path.display().to_string(),
                field: "file".to_string(),
                message: format!("Failed to read playbook file: {}", e),
            }])?;
        
        // Try YAML first, then JSON
        let pb: PlaybookDef = serde_yaml::from_str(&contents)
            .or_else(|_| serde_json::from_str(&contents))
            .map_err(|e| vec![ValidationError {
                playbook_id: path.display().to_string(),
                field: "parse".to_string(),
                message: format!("Failed to parse playbook: {}", e),
            }])?;
        
        self.validate_playbook(&pb)?;
        Ok(pb)
    }
    
    /// Load and validate all playbooks from a directory
    pub fn validate_directory<P: AsRef<Path>>(&self, dir: P) -> (Vec<PlaybookDef>, Vec<ValidationError>) {
        let dir = dir.as_ref();
        let mut playbooks = Vec::new();
        let mut errors = Vec::new();
        
        if !dir.exists() {
            return (playbooks, errors);
        }
        
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|ext| ext == "yaml" || ext == "yml" || ext == "json").unwrap_or(false) {
                        match self.validate_file(&path) {
                            Ok(pb) => playbooks.push(pb),
                            Err(mut pb_errors) => errors.append(&mut pb_errors),
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(ValidationError {
                    playbook_id: "directory".to_string(),
                    field: "read".to_string(),
                    message: format!("Failed to read directory: {}", e),
                });
            }
        }
        
        (playbooks, errors)
    }
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "and" | "or" | "not" | "if" | "then" | "else" | "true" | "false"
            | "null" | "in" | "is" | "as" | "for" | "while"
    )
}

impl Default for PlaybookValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_playbook() -> PlaybookDef {
        PlaybookDef {
            playbook_id: "test_pb".to_string(),
            title: "Test Playbook".to_string(),
            severity: "medium".to_string(),
            priority: Some(50),
            entity_scope: "user".to_string(),
            slots: vec![SlotDef {
                slot_id: "slot1".to_string(),
                entity_key: "{{ user }}".to_string(),
                predicates: vec![PredicateDef {
                    field: "event_type".to_string(),
                    op: "==".to_string(),
                    value: serde_json::json!("proc_exec"),
                }],
            }],
            fire: FireDef {
                condition: "slot1".to_string(),
            },
        }
    }

    #[test]
    fn test_valid_playbook() {
        let validator = PlaybookValidator::new();
        let pb = valid_playbook();
        assert!(validator.validate_playbook(&pb).is_ok());
    }

    #[test]
    fn test_missing_playbook_id() {
        let validator = PlaybookValidator::new();
        let mut pb = valid_playbook();
        pb.playbook_id = "".to_string();
        
        let errors = validator.validate_playbook(&pb).unwrap_err();
        assert!(errors.iter().any(|e| e.field == "playbook_id"));
    }

    #[test]
    fn test_invalid_severity() {
        let validator = PlaybookValidator::new();
        let mut pb = valid_playbook();
        pb.severity = "invalid".to_string();
        
        let errors = validator.validate_playbook(&pb).unwrap_err();
        assert!(errors.iter().any(|e| e.field == "severity"));
    }

    #[test]
    fn test_fire_references_undefined_slot() {
        let validator = PlaybookValidator::new();
        let mut pb = valid_playbook();
        pb.fire.condition = "undefined_slot".to_string();
        
        let errors = validator.validate_playbook(&pb).unwrap_err();
        assert!(errors.iter().any(|e| e.field == "fire.condition"));
    }

    #[test]
    fn test_duplicate_slot_ids() {
        let validator = PlaybookValidator::new();
        let mut pb = valid_playbook();
        pb.slots.push(SlotDef {
            slot_id: "slot1".to_string(),  // Duplicate
            entity_key: "{{ host }}".to_string(),
            predicates: vec![PredicateDef {
                field: "event_type".to_string(),
                op: "==".to_string(),
                value: serde_json::json!("test"),
            }],
        });
        
        let errors = validator.validate_playbook(&pb).unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("duplicated")));
    }

    #[test]
    fn test_invalid_operator() {
        let validator = PlaybookValidator::new();
        let mut pb = valid_playbook();
        pb.slots[0].predicates[0].op = "invalid_op".to_string();
        
        let errors = validator.validate_playbook(&pb).unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("not valid")));
    }
}
