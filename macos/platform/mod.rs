//! macOS Detection System: unified evidence-first incident compiler
//! 
//! This is the production threat detection layer for macOS.
//! It implements:
//! - Evidence-first design (all incidents tied to immutable EvidencePtr)
//! - Dual playbooks: slot-based (ground truth) + heuristics (signals)
//! - Deterministic incident compilation with step traces
//! - API handlers for REST exposure

pub mod evidence;
pub mod slot_engine;
pub mod facts;
pub mod macos_fact_extractors;
pub mod macos_fact_extractors_es;
pub mod bsm_parser;

// Temporarily disabled during refactoring to macOS Model A
// These will be refactored to work with Evidence pointers and slot engine
// pub mod playbooks;
// pub mod incidents;
// pub mod enricher;
// pub mod explainer;
// pub mod explain;
// pub mod narrative;
// pub mod api;
// pub mod db_schema;
// pub mod state_machine;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod synthetic_pipeline_tests;

// Re-export key types from existing modules
pub use evidence::{EvidencePtr, HasEvidence};
pub use slot_engine::{SlotEngine, Incident as SlotIncident, PlaybookSpec, Slot};
pub use facts::{Fact, FactType};

// TODO: Re-export playbook types once refactored
// pub use playbooks::{Playbook, PlaybookA, PlaybookB, PlaybookC, PlaybookD, PlaybookE, PlaybookF};
// pub use state_machine::{Incident, IncidentFactory, EnrichedFact, PlaybookState};
// pub use explainer::ExplanationBundle;
