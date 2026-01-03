// windows/mod.rs
// Windows sensor module
// Emits core::Event from EVTX/Sysmon/ETW/Defender sources

pub mod bookmark_manager;
pub mod capture_windows_rotating;
pub mod config;
pub mod evtlog_state;
pub mod self_test;
pub mod sensors;
pub mod wevt_bookmarks;
pub mod wevt_reader;

pub use sensors::*;
