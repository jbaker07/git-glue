//! Production-ready integration layer for EDR/SIEM ingest, mapping, correlation, and export.
//!
//! This crate provides:
//! - **Adapters**: Production-ready connectors to 10+ major EDRs (CrowdStrike, SentinelOne, etc.)
//! - **Fact/Incident Models**: Coverage-scoped evidence aggregation with confidence scoring
//! - **Capabilities Manifest**: Track what each source can/cannot provide
//! - **Correlation**: Detect contradictions and coverage gaps across sources
//! - **Adaptive Reports**: Generate reports that adapt to available data
//! - **Real-time Sync**: WebSocket server for live UI updates
//! - **PDF Export**: Production-ready PDF generation for incident reports

pub mod adapter;
pub mod adaptive_report;
pub mod auto_detect;
pub mod capabilities;
pub mod correlation;
pub mod export;
pub mod export_pdf;
pub mod fact;
pub mod ingest;
pub mod mapping;
pub mod realtime_sync;

// Re-export main types for convenience
pub use adapter::{
    Adapter,
    AdapterConfig,
    AdapterRegistry,
    AuthConfig,
    BitdefenderAdapter,
    CarbonBlackAdapter,
    CortexXDRAdapter,
    // EDR Adapters
    CrowdStrikeAdapter,
    DefenderAdapter,
    EsetAdapter,
    FieldMapping,
    McAfeeAdapter,
    NormalizedEvent,
    PaginationConfig,
    RetryConfig,
    SentinelOneAdapter,
    SophosAdapter,
    TrendMicroAdapter,
};

pub use fact::{Fact, Incident, IncidentBuilder, IncidentStatus, Severity};

pub use capabilities::{CapabilitiesManifest, CoverageRegistry, Fidelity, SourceCapabilities};

pub use adaptive_report::{generate_adaptive_report, AdaptiveReport, ReportFormat, ReportSection};

pub use correlation::analyze_correlation;

pub use realtime_sync::{RealtimeSync, RealtimeSyncServer, Subscription, SyncMessage};

pub use export_pdf::{export_incident_to_pdf, export_report_to_pdf, PdfConfig, PdfGenerator};

pub use export::export_to_file;
pub use ingest::ingest_from_sources;
pub use mapping::map_events;
