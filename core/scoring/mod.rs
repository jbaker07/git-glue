//! Trust subsystem facade.
//! 
//! This module enforces the single authority for runtime trust APIs:
//! - trust_digest_engine: primary trust scoring authority
//! - trust_hook: runtime integration point for trust events
//! 
//! All shipping binaries MUST import trust functionality through this facade,
//! never directly from attic/ or non-authority modules.

pub use crate::trust_digest_engine;
pub use crate::trust_hook::{self, TrustEvent, submit_trust_event};

// Re-export commonly-used items at the facade level for convenience
pub mod engine {
    pub use crate::trust_digest_engine::*;
}

pub mod hook {
    pub use crate::trust_hook::*;
}
