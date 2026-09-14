//! Context Compiler for Agent Core.
//!
//! Transforms Agent state, observations, and optional derived knowledge into a
//! semantic intermediate representation (`CompiledContext`).

use crate::core::knowledge_store::{
    EvidenceLink, KnowledgeClaim, KnowledgeStore, Unknown as KnowledgeUnknown,
};
use crate::core::observation_store::ObservationStore;
use crate::core::types::{Action, AgentState, Goal, Observation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContextCompileError {
    #[error("Context compilation error: {0}")]
    Compilation(String),
}

/// Semantic intermediate representation of provider context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledContext {
    pub goal: Goal,
    pub understanding: String,
    pub observations: Vec<Observation>,
    pub recent_actions: Vec<Action>,
    pub knowledge_claims: Vec<KnowledgeClaim>,
    pub evidence_links: Vec<EvidenceLink>,
    pub unknowns: Vec<String>,
    pub knowledge_unknowns: Vec<KnowledgeUnknown>,
    pub active_problems: Vec<String>,
}

/// Abstraction for compiling agent state and observations into semantic context.
pub trait ContextCompiler: std::fmt::Debug {
    fn compile(
        &self,
        state: &AgentState,
        observations: &dyn ObservationStore,
    ) -> Result<CompiledContext, ContextCompileError>;

    /// Compile with derived Knowledge / Evidence / Unknown semantic state.
    /// The default preserves compatibility for existing compiler implementations.
    fn compile_with_knowledge(
        &self,
        state: &AgentState,
        observations: &dyn ObservationStore,
        knowledge: &dyn KnowledgeStore,
    ) -> Result<CompiledContext, ContextCompileError> {
        let mut context = self.compile(state, observations)?;
        knowledge
            .validate(observations)
            .map_err(|e| ContextCompileError::Compilation(e.to_string()))?;
        context.knowledge_claims = knowledge.claims().into_iter().cloned().collect();
        context.evidence_links = knowledge.evidence().into_iter().cloned().collect();
        context.knowledge_unknowns = knowledge.unknowns().into_iter().cloned().collect();
        Ok(context)
    }
}

/// Default deterministic context compiler using recency bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultContextCompiler {
    max_observations: usize,
    max_actions: usize,
}

impl Default for DefaultContextCompiler {
    fn default() -> Self {
        Self {
            max_observations: 20,
            max_actions: 20,
        }
    }
}

impl DefaultContextCompiler {
    pub fn new(max_observations: usize, max_actions: usize) -> Self {
        Self {
            max_observations,
            max_actions,
        }
    }
}

impl ContextCompiler for DefaultContextCompiler {
    fn compile(
        &self,
        state: &AgentState,
        observations: &dyn ObservationStore,
    ) -> Result<CompiledContext, ContextCompileError> {
        state
            .goal
            .validate()
            .map_err(|e| ContextCompileError::Compilation(e.to_string()))?;

        let stored_obs = observations.recent(self.max_observations);
        let obs_vec: Vec<Observation> = stored_obs.into_iter().cloned().collect();

        let act_start = state.recent_actions.len().saturating_sub(self.max_actions);
        let act_vec: Vec<Action> = state.recent_actions[act_start..].to_vec();

        Ok(CompiledContext {
            goal: state.goal.clone(),
            understanding: state.understanding.clone(),
            observations: obs_vec,
            recent_actions: act_vec,
            knowledge_claims: Vec::new(),
            evidence_links: Vec::new(),
            unknowns: state.unknowns.clone(),
            knowledge_unknowns: Vec::new(),
            active_problems: state.active_problems.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::knowledge_store::{
        EvidenceRelation, InMemoryKnowledgeStore, KnowledgeClaimStatus,
    };
    use crate::core::observation_store::InMemoryObservationStore;
    use crate::core::types::{ActionType, ObservationKind};

    #[test]
    fn test_compiler_preserves_goal_and_understanding() {
        let goal = Goal::new("Refactor authentication module");
        let mut state = AgentState::new(goal.clone());
        state.understanding = "Auth module uses JWT tokens".to_string();

        let store = InMemoryObservationStore::new();
        let compiler = DefaultContextCompiler::default();

        let ctx = compiler.compile(&state, &store).expect("should compile");
        assert_eq!(ctx.goal, goal);
        assert_eq!(ctx.understanding, "Auth module uses JWT tokens");
        assert!(ctx.observations.is_empty());
        assert!(ctx.recent_actions.is_empty());
        assert!(ctx.knowledge_claims.is_empty());
    }

    #[test]
    fn test_compiler_includes_recent_observations_and_actions() {
        let goal = Goal::new("Test goal");
        let mut state = AgentState::new(goal);

        let action = Action::new("act-1", ActionType::Execute).with_intent("check lint");
        state.record_action(action.clone());

        let mut store = InMemoryObservationStore::new();
        let obs = Observation::new("obs-1", ObservationKind::Environment, "clean check");
        store.record(obs.clone()).unwrap();

        let compiler = DefaultContextCompiler::default();
        let ctx = compiler.compile(&state, &store).expect("should compile");

        assert_eq!(ctx.observations, vec![obs]);
        assert_eq!(ctx.recent_actions, vec![action]);
    }

    #[test]
    fn test_compiler_preserves_unknowns_and_active_problems() {
        let goal = Goal::new("Test goal");
        let mut state = AgentState::new(goal);
        state.unknowns.push("API endpoint URL".to_string());
        state.active_problems.push("Port 8080 in use".to_string());

        let store = InMemoryObservationStore::new();
        let compiler = DefaultContextCompiler::default();
        let ctx = compiler.compile(&state, &store).expect("should compile");
        assert_eq!(ctx.unknowns, vec!["API endpoint URL".to_string()]);
        assert_eq!(ctx.active_problems, vec!["Port 8080 in use".to_string()]);
    }

    #[test]
    fn test_compiler_deterministic_bounded_selection() {
        let goal = Goal::new("Test goal");
        let state = AgentState::new(goal);
        let mut store = InMemoryObservationStore::new();
        for i in 1..=10 {
            let obs = Observation::new(
                format!("obs-{}", i),
                ObservationKind::Environment,
                format!("summary {}", i),
            );
            store.record(obs).unwrap();
        }

        let compiler = DefaultContextCompiler::new(3, 3);
        let ctx = compiler.compile(&state, &store).expect("should compile");
        assert_eq!(ctx.observations.len(), 3);
        assert_eq!(ctx.observations[0].id, "obs-8");
        assert_eq!(ctx.observations[1].id, "obs-9");
        assert_eq!(ctx.observations[2].id, "obs-10");
    }

    #[test]
    fn test_store_history_independent_of_bounded_context() {
        let goal = Goal::new("Test goal");
        let state = AgentState::new(goal);
        let mut store = InMemoryObservationStore::new();
        for i in 1..=5 {
            let obs = Observation::new(
                format!("obs-{}", i),
                ObservationKind::Environment,
                format!("summary {}", i),
            );
            store.record(obs).unwrap();
        }

        let compiler = DefaultContextCompiler::new(2, 2);
        let ctx = compiler.compile(&state, &store).expect("should compile");
        assert_eq!(ctx.observations.len(), 2);
        assert_eq!(ctx.observations[0].id, "obs-4");
        assert_eq!(ctx.observations[1].id, "obs-5");
        assert_eq!(store.len(), 5);
        assert!(store.get("obs-1").is_some());
        assert!(store.get("obs-2").is_some());
    }

    #[test]
    fn test_compiler_consumes_knowledge_semantics_deterministically() {
        let goal = Goal::new("Inspect service");
        let state = AgentState::new(goal);
        let mut observations = InMemoryObservationStore::new();
        observations
            .record(Observation::new("obs-1", ObservationKind::Environment, "port open"))
            .unwrap();

        let mut knowledge = InMemoryKnowledgeStore::new();
        knowledge
            .record_claim_with_evidence(
                KnowledgeClaim {
                    id: "claim-1".into(),
                    subject: "service".into(),
                    predicate: "port".into(),
                    value: "8080".into(),
                    status: KnowledgeClaimStatus::Observed,
                    scope: "current host".into(),
                    evidence_refs: vec!["obs-1".into()],
                },
                vec![EvidenceLink {
                    observation_id: "obs-1".into(),
                    claim_id: "claim-1".into(),
                    relation: EvidenceRelation::Supports,
                }],
                &observations,
            )
            .unwrap();
        knowledge
            .record_unknown(KnowledgeUnknown {
                id: "unknown-1".into(),
                subject: "service".into(),
                scope: "current host".into(),
                question: "Which process owns the port?".into(),
            })
            .unwrap();

        let compiler = DefaultContextCompiler::default();
        let first = compiler
            .compile_with_knowledge(&state, &observations, &knowledge)
            .unwrap();
        let second = compiler
            .compile_with_knowledge(&state, &observations, &knowledge)
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.knowledge_claims.len(), 1);
        assert_eq!(first.evidence_links.len(), 1);
        assert_eq!(first.knowledge_unknowns.len(), 1);
    }

    #[test]
    fn test_identical_inputs_produce_identical_output() {
        let goal = Goal::new("Test goal");
        let state = AgentState::new(goal);
        let mut store = InMemoryObservationStore::new();
        store
            .record(Observation::new(
                "obs-1",
                ObservationKind::Environment,
                "test",
            ))
            .unwrap();

        let compiler = DefaultContextCompiler::default();
        let ctx1 = compiler.compile(&state, &store).unwrap();
        let ctx2 = compiler.compile(&state, &store).unwrap();
        assert_eq!(ctx1, ctx2);
    }
}
