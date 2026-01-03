//! EDR Forensic Hooks - Root Library
//!
//! This crate re-exports the workspace crates for unified access.
//! For new code, prefer importing from the individual crates directly:
//!
//! - `edr_core` - Core types (Event, EvidencePtr)
//! - `edr_locald` - Detection daemon (SignalOrchestrator, Pipeline)
//! - `agent_macos` - macOS agent
//! - `agent_linux` - Linux agent
//! - `agent_windows` - Windows agent
//! - `workbench` - Attack documentation workbench
//! - `edr_server` - HTTP API server

// Re-export workspace crates
pub use edr_core as core;
pub use edr_locald as locald;

// Legacy module for mock data (still used by some tests)
pub mod mock;