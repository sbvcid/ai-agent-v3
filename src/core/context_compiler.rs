//! Context Compiler for Agent Core.
//!
//! Transforms Agent state and stored observations into a semantic intermediate representation (`CompiledContext`).

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
    pub unknowns: Vec<String>,
    pub active_problems: Vec<String>,
}

/// Abstraction for compiling agent state and observations into semantic context.
pub trait ContextCompiler: std::fmt::Debug {
    fn compile(
        &self,
        state: &AgentState,
        observations: &dyn ObservationStore,
    ) -> Result<CompiledContext, ContextCompileError>;
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
            unknowns: state.unknowns.clone(),
            active_problems: state.active_problems.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        // Bounded context has only last 2 observations
        assert_eq!(ctx.observations.len(), 2);
        assert_eq!(ctx.observations[0].id, "obs-4");
        assert_eq!(ctx.observations[1].id, "obs-5");

        // Store itself still retains all 5 observations (store history is intact and unmutated)
        assert_eq!(store.len(), 5);
        assert!(store.get("obs-1").is_some());
        assert!(store.get("obs-2").is_some());
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
