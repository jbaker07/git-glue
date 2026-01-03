//! HypothesisController: Runtime wiring for the incident compiler.
//!
//! This module connects the hypothesis/incident system to the running binary.
//! It provides the entry point for:
//! - Ingesting facts from the signal pipeline
//! - Running arbitration to rank hypotheses
//! - Building explanations for the API

use crate::hypothesis::{
    ArbitrationEngine, ArbitrationResponse, CanonicalEvent, ExplanationBuilder,
    ExplanationResponse, ExplanationVisibilityState, Fact, FocusWindow, GlobalWatermark,
    HypothesisState, HypothesisStorage, InMemoryStorage, IncidentStore, LateArrivalAction,
    LateArrivalPolicy, QueryContext, ScopeKey, SessionMode, StreamWatermark,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// HypothesisController owns the hypothesis system runtime state
pub struct HypothesisController {
    /// In-memory storage for hypotheses, incidents, and events
    storage: Arc<RwLock<InMemoryStorage>>,
    /// Arbitration engine for ranking hypotheses
    arbitration_engine: ArbitrationEngine,
    /// Active hypotheses by ID
    hypotheses: HashMap<String, HypothesisState>,
    /// Incident store for promoted hypotheses
    incident_store: IncidentStore,
    /// Host ID for this controller
    host_id: String,
    /// Late arrival policy for event ingestion
    late_arrival_policy: LateArrivalPolicy,
    /// Global watermark tracking per-stream watermarks
    global_watermark: GlobalWatermark,
}

impl HypothesisController {
    /// Create a new HypothesisController
    pub fn new(host_id: impl Into<String>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(InMemoryStorage::new())),
            arbitration_engine: ArbitrationEngine::new(),
            hypotheses: HashMap::new(),
            incident_store: IncidentStore::new(),
            host_id: host_id.into(),
            late_arrival_policy: LateArrivalPolicy::default(),
            global_watermark: GlobalWatermark::new(),
        }
    }

    /// Create with custom late arrival policy
    pub fn with_late_arrival_policy(host_id: impl Into<String>, policy: LateArrivalPolicy) -> Self {
        Self {
            storage: Arc::new(RwLock::new(InMemoryStorage::new())),
            arbitration_engine: ArbitrationEngine::new(),
            hypotheses: HashMap::new(),
            incident_store: IncidentStore::new(),
            host_id: host_id.into(),
            late_arrival_policy: policy,
            global_watermark: GlobalWatermark::new(),
        }
    }

    /// Ingest a canonical event with late arrival checking
    pub fn ingest_event(&mut self, event: CanonicalEvent) -> Result<LateArrivalAction, String> {
        // Check late arrival policy BEFORE updating watermark
        let result = self
            .late_arrival_policy
            .check_event(event.timestamp, &self.global_watermark);

        match result.action {
            LateArrivalAction::ProcessNormal | LateArrivalAction::UpdateHypothesis => {
                // Update watermark and store the event
                self.update_watermark_for_event(&event);
                let mut storage = self.storage.write().map_err(|e| e.to_string())?;
                storage.store_event(&event).map_err(|e| e.to_string())?;
                Ok(result.action)
            }
            LateArrivalAction::MayReopenIncident => {
                // Late but within reopen window - store and let caller handle reopen logic
                // Don't update watermark for late events
                let mut storage = self.storage.write().map_err(|e| e.to_string())?;
                storage.store_event(&event).map_err(|e| e.to_string())?;
                Ok(result.action)
            }
            LateArrivalAction::LateEnrichmentOnly => {
                // Too late to mutate, but can still enrich
                // Don't update watermark
                let mut storage = self.storage.write().map_err(|e| e.to_string())?;
                storage.store_event(&event).map_err(|e| e.to_string())?;
                Ok(result.action)
            }
            LateArrivalAction::Reject => {
                // Event is too old, don't store it
                Ok(LateArrivalAction::Reject)
            }
        }
    }

    /// Update watermark for an accepted event
    fn update_watermark_for_event(&mut self, event: &CanonicalEvent) {
        let stream_id = event.evidence_ptr.stream_id.clone();

        // Get existing watermark or create new
        let watermark = if let Some(existing) = self.global_watermark.streams.get(&stream_id) {
            StreamWatermark {
                stream_id: stream_id.clone(),
                high_watermark: existing.high_watermark.max(event.timestamp),
                low_watermark: existing.low_watermark.min(event.timestamp),
                events_in_flight: existing.events_in_flight,
                updated_at: chrono::Utc::now(),
            }
        } else {
            StreamWatermark {
                stream_id: stream_id.clone(),
                high_watermark: event.timestamp,
                low_watermark: event.timestamp,
                events_in_flight: 0,
                updated_at: chrono::Utc::now(),
            }
        };

        self.global_watermark.update_stream(watermark);
    }

    /// Ingest a fact derived from events
    /// This is the main entry point for the signal pipeline to feed the hypothesis system
    pub fn ingest_fact(&mut self, fact: Fact) -> Result<Vec<String>, String> {
        // Store the fact
        {
            let mut storage = self.storage.write().map_err(|e| e.to_string())?;
            storage.store_fact(&fact).map_err(|e| e.to_string())?;
        }

        // Find or create hypotheses that match this fact
        let affected_hypothesis_ids = self.update_hypotheses_for_fact(&fact)?;

        Ok(affected_hypothesis_ids)
    }

    /// Update hypotheses based on a new fact
    fn update_hypotheses_for_fact(&mut self, _fact: &Fact) -> Result<Vec<String>, String> {
        // In a full implementation, this would:
        // 1. Match fact against playbook slot predicates
        // 2. Create new hypotheses if fact matches a starting slot
        // 3. Fill existing hypothesis slots if fact matches continuation slots
        // 4. Return IDs of all affected hypotheses

        // For now, return empty - will be wired to playbook matching
        Ok(Vec::new())
    }

    /// Upsert a hypothesis (create or update)
    pub fn upsert_hypothesis(&mut self, hypothesis: HypothesisState) -> Result<(), String> {
        let id = hypothesis.hypothesis_id.clone();

        // Store in storage
        {
            let mut storage = self.storage.write().map_err(|e| e.to_string())?;
            storage
                .store_hypothesis(&hypothesis)
                .map_err(|e| e.to_string())?;
        }

        // Update local cache
        self.hypotheses.insert(id, hypothesis);

        Ok(())
    }

    /// Run arbitration on all active hypotheses
    pub fn arbitrate(&self) -> ArbitrationResponse {
        let candidates: Vec<&HypothesisState> =
            self.hypotheses.values().filter(|h| h.is_active()).collect();

        self.arbitration_engine.arbitrate(
            &candidates,
            None,  // focus_window
            None,  // focus_scope
            None,  // focus_families
            false, // include_expired
        )
    }

    /// Build query context from hypothesis
    fn build_query_context(&self, hypothesis: &HypothesisState) -> QueryContext {
        QueryContext {
            mode: SessionMode::Discovery,
            focus_window: Some(FocusWindow::new(
                hypothesis.window_start_ts,
                hypothesis.window_end_ts,
            )),
            focus_entities: vec![hypothesis.scope_key.clone()],
            families_enabled: vec![hypothesis.family.clone()],
            checkpoint_ref: None,
            host_id: self.host_id.clone(),
            query_ts: Utc::now(),
        }
    }

    /// Build explanation for a hypothesis
    pub fn explain(&self, hypothesis_id: &str) -> Option<ExplanationResponse> {
        let hypothesis = self.hypotheses.get(hypothesis_id)?;

        // Build query context from hypothesis
        let ctx = self.build_query_context(hypothesis);

        // Build explanation
        let visibility_state = ExplanationVisibilityState {
            streams_present: hypothesis
                .visibility_state
                .streams_present
                .iter()
                .cloned()
                .collect(),
            streams_missing: hypothesis
                .visibility_state
                .streams_missing
                .iter()
                .cloned()
                .collect(),
            degraded: !hypothesis.visibility_state.streams_missing.is_empty(),
            degraded_reasons: Vec::new(),
        };

        let response = ExplanationBuilder::new(ctx)
            .visibility(visibility_state)
            .slot_status(hypothesis)
            .build();

        Some(response)
    }

    /// Build explanation for an incident
    pub fn explain_incident(&self, incident_id: &str) -> Option<ExplanationResponse> {
        let incident = self.incident_store.get(incident_id)?;

        // Build query context from incident
        let ctx = QueryContext {
            mode: SessionMode::Discovery,
            focus_window: Some(FocusWindow::new(incident.first_ts, incident.last_ts)),
            focus_entities: vec![incident.primary_scope_key.clone()],
            families_enabled: vec![incident.family.clone()],
            checkpoint_ref: None,
            host_id: incident.host_id.clone(),
            query_ts: Utc::now(),
        };

        // Build explanation
        let visibility_state = ExplanationVisibilityState {
            streams_present: Vec::new(),
            streams_missing: Vec::new(),
            degraded: false,
            degraded_reasons: Vec::new(),
        };

        let response = ExplanationBuilder::new(ctx)
            .visibility(visibility_state)
            .build();

        Some(response)
    }

    /// Get storage for direct queries
    pub fn storage(&self) -> Arc<RwLock<InMemoryStorage>> {
        self.storage.clone()
    }

    /// Get all active hypotheses
    pub fn active_hypotheses(&self) -> Vec<&HypothesisState> {
        self.hypotheses.values().filter(|h| h.is_active()).collect()
    }

    /// Get hypothesis by ID
    pub fn get_hypothesis(&self, id: &str) -> Option<&HypothesisState> {
        self.hypotheses.get(id)
    }

    /// Get incident by ID
    pub fn get_incident(&self, id: &str) -> Option<&crate::hypothesis::Incident> {
        self.incident_store.get(id)
    }

    /// Promote a hypothesis to an incident
    pub fn promote_to_incident(&mut self, hypothesis_id: &str) -> Result<String, String> {
        let hypothesis = self
            .hypotheses
            .get_mut(hypothesis_id)
            .ok_or_else(|| format!("Hypothesis not found: {}", hypothesis_id))?;

        // Create incident from hypothesis
        let incident = crate::hypothesis::Incident::from_hypothesis(hypothesis, &self.host_id);
        let incident_id = incident.incident_id.clone();

        // Mark hypothesis as promoted
        hypothesis.promote(&incident_id);

        // Store incident
        self.incident_store.upsert(incident);

        // Update hypothesis in storage
        {
            let mut storage = self.storage.write().map_err(|e| e.to_string())?;
            storage
                .update_hypothesis(hypothesis)
                .map_err(|e| e.to_string())?;
        }

        Ok(incident_id)
    }
}

impl Default for HypothesisController {
    fn default() -> Self {
        Self::new("default_host")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypothesis::{EvidencePtr, FactType};

    #[test]
    fn test_controller_creation() {
        let controller = HypothesisController::new("test_host");
        assert!(controller.active_hypotheses().is_empty());
    }

    #[test]
    fn test_fact_ingestion() {
        let mut controller = HypothesisController::new("test_host");

        let fact = Fact::new(
            "host1",
            ScopeKey::Process {
                key: "proc_key_1".to_string(),
            },
            FactType::Exec {
                exe_hash: Some("abc123".to_string()),
                path: "/usr/bin/test".to_string(),
                signer: None,
                cmdline: Some("test --arg".to_string()),
            },
            vec![EvidencePtr::new("test_stream", "seg_001", 0)],
        );

        let result = controller.ingest_fact(fact);
        assert!(result.is_ok());
    }

    #[test]
    fn test_arbitration_empty() {
        let controller = HypothesisController::new("test_host");
        let response = controller.arbitrate();
        assert!(response.top3.is_empty());
    }
}
