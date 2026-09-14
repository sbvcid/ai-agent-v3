//! Stage 5: Deterministic Closed-Loop Agent & Event Trace.
//!
//! Provides a minimal, deterministic, purely in-memory Agent Loop that orchestrates:
//! ```text
//! AgentState
//!     ↓
//! Mock LLM
//!     ↓
//! AgentDecision
//!     ↓
//! Action
//!     ↓
//! Fake Runtime
//!     ↓
//! ActionResult
//!     ↓
//! Observation
//!     ↓
//! ObservationStore
//!     ↓
//! Semantic Update Producer / Boundary
//!     ↓
//! KnowledgeStore
//!     ↓
//! AgentState update
//!     ↓
//! next loop / Finish
//! ```
//!
//! Maintains canonical boundaries:
//! - Uses only canonical `Action`, `ActionResult`, and `Observation`.
//! - No `ToolCall` or `ToolResult`.
//! - Purely in-memory, deterministic, no OS or network side effects.

use crate::core::checkpoint::StateCheckpoint;
use crate::core::decision_source::{DecisionSource, DecisionSourceError};
use crate::core::knowledge_store::InMemoryKnowledgeStore;
use crate::core::observation_store::{
    InMemoryObservationStore, ObservationStore, ObservationStoreError,
};
use crate::core::runtime::Runtime;
use crate::core::semantic_updater::{
    KnowledgeStoreSemanticUpdater, NoOpSemanticUpdateProducer, SemanticUpdateError,
    SemanticUpdateProducer, SemanticUpdater,
};
use crate::core::types::{
    Action, ActionResult, AgentDecision, AgentState, ExecutionState, FinalTaskStatus, Observation,
    ObservationKind, ValidationError, VerificationState,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Loop Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LoopError {
    #[error("Maximum step limit ({0}) exceeded")]
    MaxStepsExceeded(usize),

    #[error("AgentState validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("Observation store error: {0}")]
    ObservationStore(#[from] ObservationStoreError),

    #[error("Semantic update error: {0}")]
    SemanticUpdate(#[from] SemanticUpdateError),

    #[error("Cannot step loop: task already finalized with status: {0:?}")]
    AlreadyFinished(FinalTaskStatus),

    #[error(
        "Cannot finish task as Done: Goal has not been verified (verification state is {0:?})"
    )]
    UnverifiedGoal(VerificationState),

    #[error("Decision source error: {0}")]
    DecisionSource(#[from] DecisionSourceError),
}

// ---------------------------------------------------------------------------
// Event Trace
// ---------------------------------------------------------------------------

/// Individual lifecycle event recorded during loop execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopEvent {
    DecisionProduced(AgentDecision),
    ActionExecuted(Action),
    ActionResultReceived(ActionResult),
    ObservationProduced(Observation),
    StateUpdated,
    GoalVerified(String),
    WaitingEntered { reason: Option<String> },
    Finished(FinalTaskStatus),
}

/// Deterministic in-memory event trace of loop execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EventTrace {
    events: Vec<LoopEvent>,
}

impl EventTrace {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: LoopEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[LoopEvent] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

// ---------------------------------------------------------------------------
// Step Outcome
// ---------------------------------------------------------------------------

/// Outcome of a single step in the Agent Loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopStepOutcome {
    /// Step completed; loop should continue with next iteration.
    Continue,
    /// Agent transitioned to Waiting state.
    Waiting,
    /// Agent transitioned to finished with a final task status.
    Finished(FinalTaskStatus),
}

// ---------------------------------------------------------------------------
// Agent Loop
// ---------------------------------------------------------------------------

/// Minimal deterministic Agent Loop.
pub struct AgentLoop {
    state: AgentState,
    observation_store: InMemoryObservationStore,
    knowledge_store: InMemoryKnowledgeStore,
    semantic_update_producer: Box<dyn SemanticUpdateProducer>,
    semantic_updater: Box<dyn SemanticUpdater>,
    trace: EventTrace,
    max_steps: usize,
    current_step: usize,
}

impl AgentLoop {
    /// Create a new AgentLoop with default step limit (50).
    pub fn new(state: AgentState) -> Self {
        let mut observation_store = InMemoryObservationStore::new();
        for obs in &state.recent_observations {
            observation_store
                .record(obs.clone())
                .expect("Failed to record observation into authoritative store");
        }
        Self {
            state,
            observation_store,
            knowledge_store: InMemoryKnowledgeStore::new(),
            semantic_update_producer: Box::new(NoOpSemanticUpdateProducer),
            semantic_updater: Box::new(KnowledgeStoreSemanticUpdater::new()),
            trace: EventTrace::new(),
            max_steps: 50,
            current_step: 0,
        }
    }

    /// Create an AgentLoop from an existing StateCheckpoint, restoring state and execution cursor.
    ///
    /// Per `docs/02` §81, Checkpoint preserves AgentState, Goal, Knowledge States, and Event Cursor.
    /// B2-A does not add KnowledgeStore persistence; the authoritative KnowledgeStore therefore
    /// starts empty until a later semantic persistence boundary is explicitly designed.
    pub fn from_checkpoint(checkpoint: &StateCheckpoint) -> Self {
        let mut observation_store = InMemoryObservationStore::new();
        for obs in &checkpoint.state.recent_observations {
            observation_store
                .record(obs.clone())
                .expect("Failed to record observation into authoritative store from checkpoint");
        }
        Self {
            state: checkpoint.state.clone(),
            observation_store,
            knowledge_store: InMemoryKnowledgeStore::new(),
            semantic_update_producer: Box::new(NoOpSemanticUpdateProducer),
            semantic_updater: Box::new(KnowledgeStoreSemanticUpdater::new()),
            trace: EventTrace::new(),
            max_steps: 50,
            current_step: checkpoint.step_index as usize,
        }
    }

    /// Configure the Core semantic producer and mutation boundary.
    ///
    /// The producer determines whether a recorded Observation warrants a semantic update;
    /// the updater validates and applies that update to the authoritative KnowledgeStore.
    /// AgentLoop remains orchestration code and does not infer Knowledge itself.
    pub fn with_semantic_update_pipeline<P, U>(mut self, producer: P, updater: U) -> Self
    where
        P: SemanticUpdateProducer + 'static,
        U: SemanticUpdater + 'static,
    {
        self.semantic_update_producer = Box::new(producer);
        self.semantic_updater = Box::new(updater);
        self
    }

    /// Immutable reference to the ObservationStore.
    pub fn observation_store(&self) -> &InMemoryObservationStore {
        &self.observation_store
    }

    /// Mutable reference to the ObservationStore.
    pub fn observation_store_mut(&mut self) -> &mut InMemoryObservationStore {
        &mut self.observation_store
    }

    /// Immutable reference to the authoritative KnowledgeStore.
    pub fn knowledge_store(&self) -> &InMemoryKnowledgeStore {
        &self.knowledge_store
    }

    /// Mutable reference to the authoritative KnowledgeStore.
    pub fn knowledge_store_mut(&mut self) -> &mut InMemoryKnowledgeStore {
        &mut self.knowledge_store
    }

    /// Record verification evidence and mark the goal as Verified.
    ///
    /// Per `docs/02` §51-53, Goal completion requires verification evidence connecting
    /// observations to the original user goal (Action Success != Goal Success).
    pub fn verify_goal(&mut self, evidence: impl Into<String>) {
        let ev = evidence.into();
        self.trace.push(LoopEvent::GoalVerified(ev.clone()));
        self.state.verify_goal(ev);
    }

    /// Configure maximum step limit as a safety boundary.
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Immutable reference to the current working memory.
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// Mutable reference to the working memory (e.g. for inspection or hypothesis revision).
    pub fn state_mut(&mut self) -> &mut AgentState {
        &mut self.state
    }

    /// Immutable reference to the recorded event trace.
    pub fn trace(&self) -> &EventTrace {
        &self.trace
    }

    /// Current step count.
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Apply a semantic update produced for an Observation that is already stored authoritatively.
    fn apply_semantic_update(&mut self, observation: &Observation) -> Result<(), LoopError> {
        let Some(update) = self.semantic_update_producer.produce(observation) else {
            return Ok(());
        };

        self.semantic_updater
            .apply(update, &self.observation_store, &mut self.knowledge_store)?;
        Ok(())
    }

    /// Execute a single step in the loop.
    pub fn step<DS: DecisionSource, R: Runtime>(
        &mut self,
        decision_source: &mut DS,
        runtime: &mut R,
    ) -> Result<LoopStepOutcome, LoopError> {
        // Prevent stepping if already finalized
        if let Some(status) = &self.state.final_status {
            return Err(LoopError::AlreadyFinished(status.clone()));
        }

        // Enforce max step bound
        if self.current_step >= self.max_steps {
            return Err(LoopError::MaxStepsExceeded(self.max_steps));
        }

        self.current_step += 1;

        // 1. Decision acquisition via DecisionSource
        let decision = decision_source.next_decision_with_stores(
            &self.state,
            &self.observation_store,
            &self.knowledge_store,
        )?;
        self.trace
            .push(LoopEvent::DecisionProduced(decision.clone()));

        // 2. Process Decision
        let outcome = match decision {
            AgentDecision::Act {
                action,
                intent,
                expected_progress,
            } => {
                let _ = (intent, expected_progress);
                self.trace.push(LoopEvent::ActionExecuted(action.clone()));

                // 3. Runtime execution
                let action_result = runtime.execute(action.clone());
                self.trace
                    .push(LoopEvent::ActionResultReceived(action_result.clone()));

                // 4. Observation produced
                let obs_id = format!("obs-{}", self.current_step);
                let observation = Observation::from_action_result(obs_id, &action_result);
                self.trace
                    .push(LoopEvent::ObservationProduced(observation.clone()));

                // 5. Observation becomes authoritative history before any semantic update.
                self.observation_store.record(observation.clone())?;
                self.state.record_action(action);
                self.state.record_observation(observation.clone());
                self.trace.push(LoopEvent::StateUpdated);

                // 6. Semantic update is derived only after ObservationStore::record succeeds.
                self.apply_semantic_update(&observation)?;

                LoopStepOutcome::Continue
            }

            AgentDecision::Observe { intent } => {
                let obs_id = format!("obs-{}", self.current_step);
                let summary = intent.unwrap_or_else(|| "Observation".to_string());
                let observation = Observation::new(obs_id, ObservationKind::Environment, summary);
                self.trace
                    .push(LoopEvent::ObservationProduced(observation.clone()));

                self.observation_store.record(observation.clone())?;
                self.state.record_observation(observation.clone());
                self.trace.push(LoopEvent::StateUpdated);

                self.apply_semantic_update(&observation)?;

                LoopStepOutcome::Continue
            }

            AgentDecision::Wait { reason } => {
                self.state.execution_state = ExecutionState::Waiting;
                self.trace.push(LoopEvent::WaitingEntered { reason });
                LoopStepOutcome::Waiting
            }

            AgentDecision::Finish { status, message } => {
                let _ = message;
                // Goal Verification required before Done (docs/02 §51-53: Action Success != Goal Success)
                if status == FinalTaskStatus::Done
                    && self.state.verification_state != VerificationState::Verified
                {
                    return Err(LoopError::UnverifiedGoal(self.state.verification_state));
                }
                self.state.finish(status.clone());
                self.trace.push(LoopEvent::Finished(status.clone()));
                LoopStepOutcome::Finished(status)
            }
        };

        // 7. Validate state invariant after transition
        self.state.validate()?;

        Ok(outcome)
    }

    /// Run the loop to completion (until Finished, Waiting, or error).
    pub fn run<DS: DecisionSource, R: Runtime>(
        &mut self,
        decision_source: &mut DS,
        runtime: &mut R,
    ) -> Result<&AgentState, LoopError> {
        loop {
            match self.step(decision_source, runtime)? {
                LoopStepOutcome::Continue => continue,
                LoopStepOutcome::Waiting => break,
                LoopStepOutcome::Finished(_) => break,
            }
        }
        Ok(&self.state)
    }
}

#[cfg(test)]
mod semantic_integration_tests {
    use super::*;
    use crate::core::knowledge_store::{
        EvidenceLink, EvidenceRelation, KnowledgeClaim, KnowledgeClaimStatus, KnowledgeStore,
    };
    use crate::core::semantic_updater::SemanticUpdate;
    use crate::core::test_doubles::{FakeRuntime, MockLlm};
    use crate::core::types::{ActionType, Goal};

    #[derive(Debug)]
    struct FixedClaimProducer;

    impl SemanticUpdateProducer for FixedClaimProducer {
        fn produce(&mut self, observation: &Observation) -> Option<SemanticUpdate> {
            let observation_id = observation.id.clone();
            let action_id = observation
                .source_action_id
                .clone()
                .unwrap_or_else(|| "unknown-action".to_string());
            let claim_id = format!("claim-for-{}", observation_id);
            Some(SemanticUpdate::ClaimWithEvidence {
                claim: KnowledgeClaim {
                    id: claim_id.clone(),
                    subject: action_id,
                    predicate: "result".into(),
                    value: observation.summary.clone(),
                    status: KnowledgeClaimStatus::Observed,
                    scope: "current task".into(),
                    evidence_refs: vec![observation_id.clone()],
                },
                evidence: vec![EvidenceLink {
                    observation_id,
                    claim_id,
                    relation: EvidenceRelation::Supports,
                }],
            })
        }
    }

    #[test]
    fn agent_loop_records_observation_before_semantic_update() {
        let state = AgentState::new(Goal::new("test semantic integration"));
        let mut agent = AgentLoop::new(state).with_semantic_update_pipeline(
            FixedClaimProducer,
            KnowledgeStoreSemanticUpdater::new(),
        );
        let action = Action::new("act-1", ActionType::Execute);
        let mut decision_source = MockLlm::with_decisions(vec![AgentDecision::act(action)]);
        let mut runtime = FakeRuntime::new();

        agent.step(&mut decision_source, &mut runtime).unwrap();

        let observation = agent
            .observation_store()
            .get("obs-1")
            .expect("Observation must be stored before semantic update");
        assert_eq!(observation.source_action_id.as_deref(), Some("act-1"));
        assert_eq!(agent.knowledge_store().claims().len(), 1);
        assert_eq!(agent.knowledge_store().evidence().len(), 1);
        assert!(agent
            .knowledge_store()
            .evidence()
            .iter()
            .any(|link| link.observation_id == "obs-1"));
    }

    #[derive(Debug)]
    struct InvalidEvidenceProducer;

    impl SemanticUpdateProducer for InvalidEvidenceProducer {
        fn produce(&mut self, _observation: &Observation) -> Option<SemanticUpdate> {
            Some(SemanticUpdate::Evidence(EvidenceLink {
                observation_id: "missing-observation".into(),
                claim_id: "missing-claim".into(),
                relation: EvidenceRelation::Supports,
            }))
        }
    }

    #[test]
    fn semantic_update_failure_preserves_recorded_observation() {
        let state = AgentState::new(Goal::new("semantic failure"));
        let mut agent = AgentLoop::new(state).with_semantic_update_pipeline(
            InvalidEvidenceProducer,
            KnowledgeStoreSemanticUpdater::new(),
        );
        let action = Action::new("act-1", ActionType::Execute);
        let mut decision_source = MockLlm::with_decisions(vec![AgentDecision::act(action)]);
        let mut runtime = FakeRuntime::new();

        let error = agent
            .step(&mut decision_source, &mut runtime)
            .expect_err("invalid semantic update must fail deterministically");
        assert!(matches!(error, LoopError::SemanticUpdate(_)));
        assert!(agent.observation_store().get("obs-1").is_some());
        assert!(agent.knowledge_store().claims().is_empty());
        assert!(agent.knowledge_store().evidence().is_empty());
        assert!(agent.knowledge_store().unknowns().is_empty());
    }
}
