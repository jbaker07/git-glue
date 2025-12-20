// GNN module stub: moved to attic/rust_deadcode/src/gnn/
// This stub exists to prevent import errors in code that references gnn modules.
// For "NO GNN in shipping lanes", all gnn hook calls are disabled.

pub mod hook {
    use crate::telemetry::TelemetryRecord;

    /// No-op stub: GNN hooks disabled
    pub fn push_to_gnn_vector_log(_record: impl Clone) {
        // GNN functionality removed from shipping lanes
    }

    pub fn submit_to_gnn(_record: &TelemetryRecord) {
        // GNN functionality removed from shipping lanes
    }
}

pub mod bridge {
    // GNN bridge stub (unused in Tier-0 lanes)
}

pub mod models {
    // GNN models stub (unused in Tier-0 lanes)
}
