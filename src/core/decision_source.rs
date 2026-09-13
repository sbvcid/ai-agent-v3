//! Stage 9.2A — Provider-independent DecisionSource abstraction.
//!
//! Defines the [`DecisionSource`] trait decoupling [`AgentLoop`] from specific
//! LLM provider implementations (such as `MockLlm` or future provider adapters).

use crate::core::types::{AgentDecision, AgentState};
use thiserror::Error;

/// Errors produced when obtaining a decision from a [`DecisionSource`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DecisionSourceError {
    #[error("Decision source exhausted")]
    Exhausted,

    #[error("Decision source provider error: {0}")]
    Provider(String),

    #[error("Decision source error: {0}")]
    Other(String),
}

/// Provider-independent decision source abstraction.
///
/// Permits the Agent Loop to obtain decisions from any source (mock, LLM provider, etc.)
/// given the current working state of the agent.
pub trait DecisionSource {
    fn next_decision(&mut self, state: &AgentState) -> Result<AgentDecision, DecisionSourceError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::agent_loop::{AgentLoop, LoopStepOutcome};
    use crate::core::test_doubles::{FakeRuntime, MockLlm};
    use crate::core::types::{Action, ActionType, FinalTaskStatus, Goal};

    #[test]
    fn test_mock_llm_as_decision_source() {
        let mut mock = MockLlm::with_decisions(vec![
            AgentDecision::observe("check state"),
            AgentDecision::finish(FinalTaskStatus::Done),
        ]);
        let state = AgentState::new(Goal::new("test goal"));

        // 1. First decision via DecisionSource trait
        let d1 = DecisionSource::next_decision(&mut mock, &state).expect("should succeed");
        assert!(matches!(d1, AgentDecision::Observe { .. }));

        // 2. Second decision via DecisionSource trait
        let d2 = DecisionSource::next_decision(&mut mock, &state).expect("should succeed");
        assert!(matches!(
            d2,
            AgentDecision::Finish {
                status: FinalTaskStatus::Done,
                ..
            }
        ));

        // 3. Exhausted deterministic fallback via DecisionSource trait
        let d3 = DecisionSource::next_decision(&mut mock, &state).expect("should succeed");
        assert!(matches!(
            d3,
            AgentDecision::Finish {
                status: FinalTaskStatus::Blocked { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_agent_loop_execution_via_decision_source() {
        let goal = Goal::new("Test loop with DecisionSource");
        let state = AgentState::new(goal);

        let mut mock = MockLlm::with_decisions(vec![AgentDecision::act(
            Action::new("act-1", ActionType::Execute)
                .with_parameter("executable", "echo")
                .with_intent("Run echo"),
        )]);

        let mut runtime = FakeRuntime::new();
        runtime.add_result(
            "act-1",
            crate::core::ActionResult::success("act-1", "hello"),
        );

        let mut agent_loop = AgentLoop::new(state);
        let outcome = agent_loop
            .step(&mut mock, &mut runtime)
            .expect("step must succeed");
        assert_eq!(outcome, LoopStepOutcome::Continue);
        assert_eq!(agent_loop.current_step(), 1);
    }
}
