// macos/sensors/es/mod.rs
// Endpoint Security (ES) event extractor
// Converts ES events to canonical core::Event

pub mod es_client;
pub mod file_create;
pub mod file_metadata;
pub mod file_open;
pub mod file_rename;
pub mod file_unlink;
pub mod file_write;
pub mod identity;
pub mod mounts;
pub mod proc_exec;
pub mod proc_lifecycle;

pub use es_client::ESClient;
