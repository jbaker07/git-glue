//! GNN subsystem facade.
//!
//! This module enforces the single authority for runtime GNN APIs:
//! - gnn_hook: runtime integration point for GNN events
//! - gnn_models: model definitions and scoring logic
//!
//! All shipping binaries MUST import GNN functionality through this facade,
//! never from attic/ or deprecated variant modules.

pub use crate::gnn_hook::{self, push_to_gnn_vector_log, submit_to_gnn};
pub use crate::gnn_models;

// Re-export commonly-used items at the facade level for convenience
pub mod hook {
    pub use crate::gnn_hook::*;
}

pub mod models {
    pub use crate::gnn_models::*;
}
