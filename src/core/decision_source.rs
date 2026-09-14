//! Stage 9.2A — Provider-independent DecisionSource abstraction.
//!
//! Defines the [`DecisionSource`] trait decoupling [`AgentLoop`] from specific
//! LLM provider implementations (such as `MockLlm` or future provider adapters).

use crate::core::context_compiler::{CompiledContext, ContextCompiler, DefaultContextCompiler};
use crate::core::interpreter::interpret;
use crate::core::observation_store::{InMemoryObservationStore, ObservationStore};
use crate::core::types::{AgentDecision, AgentState};
use crate::provider::{LlmProvider, ProviderMessage, ProviderRequest};
use thiserror::Error;

/// Errors produced when obtaining a decision from a [`DecisionSource`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DecisionSourceError {
    #[error("Decision source exhausted")]
    Exhausted,

    #[error("Decision source provider error: {0}")]
    Provider(String),

    #[error("Decision source interpretation error: {0}")]
    Interpretation(String),

    #[error("Decision source error: {0}")]
    Other(String),
}

/// Provider-independent decision source abstraction.
///
/// Permits the Agent Loop to obtain decisions from any source (mock, LLM provider, etc.)
/// given the current working state of the agent.
pub trait DecisionSource {
    fn next_decision(&mut self, state: &AgentState) -> Result<AgentDecision, DecisionSourceError>;

    fn next_decision_with_store(
        &mut self,
        state: &AgentState,
        _store: &dyn ObservationStore,
    ) -> Result<AgentDecision, DecisionSourceError> {
        self.next_decision(state)
    }
}

/// Provider-backed DecisionSource connecting an [`LlmProvider`] and the core interpreter.
pub struct ProviderDecisionSource<P>
where
    P: LlmProvider,
{
    provider: P,
    compiler: DefaultContextCompiler,
}

impl<P> ProviderDecisionSource<P>
where
    P: LlmProvider,
{
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            compiler: DefaultContextCompiler::default(),
        }
    }

    pub fn provider(&self) -> &P {
        &self.provider
    }

    pub fn provider_mut(&mut self) -> &mut P {
        &mut self.provider
    }
}

impl<P> DecisionSource for ProviderDecisionSource<P>
where
    P: LlmProvider,
{
    fn next_decision(&mut self, state: &AgentState) -> Result<AgentDecision, DecisionSourceError> {
        let mut temp_store = InMemoryObservationStore::new();
        for obs in &state.recent_observations {
            temp_store
                .record(obs.clone())
                .map_err(|e| DecisionSourceError::Other(e.to_string()))?;
        }
        self.next_decision_with_store(state, &temp_store)
    }

    fn next_decision_with_store(
        &mut self,
        state: &AgentState,
        store: &dyn ObservationStore,
    ) -> Result<AgentDecision, DecisionSourceError> {
        let compiled = self
            .compiler
            .compile(state, store)
            .map_err(|e| DecisionSourceError::Other(e.to_string()))?;
        let request = ProviderRequest::from_compiled_context(&compiled);
        let response = self
            .provider
            .chat(&request)
            .map_err(|e| DecisionSourceError::Provider(e.to_string()))?;
        let decision =
            interpret(&response).map_err(|e| DecisionSourceError::Interpretation(e.to_string()))?;
        Ok(decision)
    }
}

impl ProviderRequest {
    /// Construct a provider-neutral chat completion request from a [`CompiledContext`].
    pub fn from_compiled_context(compiled: &CompiledContext) -> Self {
        let mut content = format!("Goal: {}\n", compiled.goal.description);
        if !compiled.understanding.is_empty() {
            content.push_str(&format!("Understanding: {}\n", compiled.understanding));
        }
        if !compiled.observations.is_empty() {
            content.push_str("Recent Observations:\n");
            for obs in &compiled.observations {
                content.push_str(&format!("- [{:?}] {}\n", obs.kind, obs.summary));
            }
        }
        if !compiled.recent_actions.is_empty() {
            content.push_str("Recent Actions:\n");
            for action in &compiled.recent_actions {
                content.push_str(&format!(
                    "- ID: {}, Type: {:?}, Intent: {:?}\n",
                    action.id, action.action_type, action.intent
                ));
            }
        }
        if !compiled.unknowns.is_empty() {
            content.push_str("Unknowns:\n");
            for unk in &compiled.unknowns {
                content.push_str(&format!("- {}\n", unk));
            }
        }
        if !compiled.active_problems.is_empty() {
            content.push_str("Active Problems:\n");
            for prob in &compiled.active_problems {
                content.push_str(&format!("- {}\n", prob));
            }
        }
        Self {
            messages: vec![ProviderMessage::User(content)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::agent_loop::{AgentLoop, LoopStepOutcome};
    use crate::core::test_doubles::{FakeRuntime, MockLlm};
    use crate::core::types::{
        Action, ActionResult, ActionType, FinalTaskStatus, Goal, Observation, ObservationKind,
    };
    use crate::provider::{ProviderError, ProviderMessage, ProviderResponse};

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

    #[test]
    fn test_provider_decision_source_success_act() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![crate::provider::ProviderToolCall {
                id: "tc-exec".to_string(),
                name: "execute_process".to_string(),
                arguments: serde_json::json!({ "executable": "cargo", "args": ["test"] }),
            }],
            finish_reason: Some("tool_calls".to_string()),
        };
        let provider =
            crate::core::test_doubles::FakeLlmProvider::with_responses(vec![Ok(response)]);
        let mut source = ProviderDecisionSource::new(provider);
        let state = AgentState::new(Goal::new("Run cargo test"));

        let decision = DecisionSource::next_decision(&mut source, &state).expect("should succeed");
        match decision {
            AgentDecision::Act { action, .. } => {
                assert_eq!(action.id, "tc-exec");
                assert_eq!(action.action_type, ActionType::Execute);
                assert_eq!(action.parameters.get("executable").unwrap(), "cargo");
            }
            other => panic!("Expected Act decision, got: {:?}", other),
        }

        assert_eq!(source.provider().request_count(), 1);
    }

    #[test]
    fn test_provider_decision_source_read_file() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![crate::provider::ProviderToolCall {
                id: "tc-read".to_string(),
                name: "read_file".to_string(),
                arguments: serde_json::json!({ "path": "src/lib.rs" }),
            }],
            finish_reason: None,
        };
        let provider =
            crate::core::test_doubles::FakeLlmProvider::with_responses(vec![Ok(response)]);
        let mut source = ProviderDecisionSource::new(provider);
        let state = AgentState::new(Goal::new("Read lib.rs"));

        let decision = DecisionSource::next_decision(&mut source, &state).expect("should succeed");
        match decision {
            AgentDecision::Act { action, .. } => {
                assert_eq!(action.id, "tc-read");
                assert_eq!(action.action_type, ActionType::Observe);
                assert_eq!(action.parameters.get("op").unwrap(), "read_file");
                assert_eq!(action.parameters.get("path").unwrap(), "src/lib.rs");
            }
            other => panic!("Expected Act decision, got: {:?}", other),
        }
    }

    #[test]
    fn test_provider_decision_source_provider_failure() {
        let provider = crate::core::test_doubles::FakeLlmProvider::with_responses(vec![Err(
            ProviderError::Unavailable("Ollama down".to_string()),
        )]);
        let mut source = ProviderDecisionSource::new(provider);
        let state = AgentState::new(Goal::new("Fail test"));

        let err = DecisionSource::next_decision(&mut source, &state).expect_err("should fail");
        assert!(matches!(
            err,
            DecisionSourceError::Provider(ref msg) if msg.contains("Ollama down")
        ));
    }

    #[test]
    fn test_provider_decision_source_interpretation_failure() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![crate::provider::ProviderToolCall {
                id: "tc-bad".to_string(),
                name: "unsupported_tool_name".to_string(),
                arguments: serde_json::json!({}),
            }],
            finish_reason: None,
        };
        let provider =
            crate::core::test_doubles::FakeLlmProvider::with_responses(vec![Ok(response)]);
        let mut source = ProviderDecisionSource::new(provider);
        let state = AgentState::new(Goal::new("Interpretation failure test"));

        let err = DecisionSource::next_decision(&mut source, &state).expect_err("should fail");
        assert!(matches!(
            err,
            DecisionSourceError::Interpretation(ref msg) if msg.contains("unsupported_tool_name")
        ));
    }

    #[test]
    fn test_provider_request_from_compiled_context() {
        let goal = Goal::new("Refactor codebase");
        let mut state = AgentState::new(goal);
        state.understanding = "Code is modular".to_string();
        state.record_action(Action::new("act-1", ActionType::Execute).with_intent("check lint"));

        let mut store = InMemoryObservationStore::new();
        store
            .record(Observation::new(
                "obs-1",
                ObservationKind::Environment,
                "clean status",
            ))
            .unwrap();

        let compiler = DefaultContextCompiler::default();
        let compiled = compiler.compile(&state, &store).unwrap();
        let req = ProviderRequest::from_compiled_context(&compiled);
        assert!(!req.messages.is_empty());
        match &req.messages[0] {
            ProviderMessage::User(text) => {
                assert!(text.contains("Refactor codebase"));
                assert!(text.contains("Code is modular"));
                assert!(text.contains("clean status"));
                assert!(text.contains("act-1"));
            }
            other => panic!("Expected User message, got: {:?}", other),
        }
    }

    #[test]
    fn test_compiler_and_provider_decision_source_accesses_authoritative_store_absent_from_state() {
        let goal = Goal::new("Test authoritative store vs state");
        let state = AgentState::new(goal); // recent_observations is empty

        let mut store = InMemoryObservationStore::new();
        let obs = Observation::new(
            "obs-auth",
            ObservationKind::Environment,
            "authoritative observation",
        );
        store.record(obs.clone()).unwrap();

        let compiler = DefaultContextCompiler::default();
        let compiled = compiler.compile(&state, &store).unwrap();
        assert_eq!(compiled.observations, vec![obs]);
        assert!(state.recent_observations.is_empty());

        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![crate::provider::ProviderToolCall {
                id: "tc-1".to_string(),
                name: "execute_process".to_string(),
                arguments: serde_json::json!({ "executable": "echo", "args": ["hello"] }),
            }],
            finish_reason: None,
        };
        let provider =
            crate::core::test_doubles::FakeLlmProvider::with_responses(vec![Ok(response)]);
        let mut source = ProviderDecisionSource::new(provider);

        let decision = source
            .next_decision_with_store(&state, &store)
            .expect("should succeed");
        assert!(matches!(decision, AgentDecision::Act { .. }));
    }

    #[test]
    fn test_provider_decision_source_compatibility_with_agent_loop() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![crate::provider::ProviderToolCall {
                id: "tc-1".to_string(),
                name: "execute_process".to_string(),
                arguments: serde_json::json!({ "executable": "echo", "args": ["hello"] }),
            }],
            finish_reason: None,
        };
        let provider =
            crate::core::test_doubles::FakeLlmProvider::with_responses(vec![Ok(response)]);
        let mut source = ProviderDecisionSource::new(provider);
        let state = AgentState::new(Goal::new("Run echo via loop"));

        let mut runtime = FakeRuntime::new();
        runtime.add_result("tc-1", ActionResult::success("tc-1", "hello"));

        let mut agent_loop = AgentLoop::new(state);
        let outcome = agent_loop
            .step(&mut source, &mut runtime)
            .expect("step must succeed");
        assert_eq!(outcome, LoopStepOutcome::Continue);
        assert_eq!(agent_loop.current_step(), 1);
        assert_eq!(agent_loop.state().recent_actions.len(), 1);
        assert_eq!(agent_loop.state().recent_actions[0].id, "tc-1");
    }
}
