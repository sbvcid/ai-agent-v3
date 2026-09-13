use crate::core::agent_loop::AgentLoop;
use crate::core::test_doubles::{FakeRuntime, MockLlm};
use crate::core::types::{Action, ActionType, AgentDecision, AgentState, FinalTaskStatus, Goal};
use crate::core::decision_source::{DecisionSource, DecisionSourceError};
use crate::core::types::ExecutionState;
use crate::provider::{ProviderError, ProviderRequest, ProviderResponse, ProviderToolCall};


use serde_json::json;

/// Mock decision source for testing
#[derive(Debug, Default)]
pub struct MockDecisionSource {
    /// Scripted decisions to return in sequence
    pub decisions: Vec<AgentDecision>,
    /// Current index in the decisions vector
    pub index: usize,
    /// Whether to return an error on the next call
    pub return_error: bool,
    /// Error to return if return_error is true
    pub error_message: String,
}

impl MockDecisionSource {
    /// Create a new mock decision source with the given decisions
    pub fn new(decisions: Vec<AgentDecision>) -> Self {
        Self {
            decisions,
            index: 0,
            return_error: false,
            error_message: String::new(),
        }
    }

    /// Configure the mock to return an error on the next call
    pub fn expect_error(&mut self, message: impl Into<String>) {
        self.return_error = true;
        self.error_message = message.into();
    }
}

impl DecisionSource for MockDecisionSource {
    fn next_decision(&mut self, _state: &AgentState) -> Result<AgentDecision, DecisionSourceError> {
        if self.return_error {
            self.return_error = false;
            return Err(DecisionSourceError::GenerationFailed(self.error_message.clone()));
        }

        if self.index >= self.decisions.len() {
            panic!("MockDecisionSource exhausted: no more decisions available");
        }

        let decision = self.decisions[self.index].clone();
        self.index += 1;
        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{VerificationState, FinalTaskStatus};

    #[test]
    fn test_mock_decision_source() {
        let mut source = MockDecisionSource::new(vec![
            AgentDecision::Act {
                action: Action::create_test(),
                intent: Some("test intent".to_string()),
                expected_progress: Some("test progress".to_string()),
            },
            AgentDecision::Finish {
                status: FinalTaskStatus::Done,
                message: Some("test finished".to_string()),
            },
        ]);

        let state = AgentState::default();
        assert_eq!(
            source.next_decision(&state).unwrap(),
            AgentDecision::Act {
                action: Action::create_test(),
                intent: Some("test intent".to_string()),
                expected_progress: Some("test progress".to_string()),
            }
        );

        assert_eq!(
            source.next_decision(&state).unwrap(),
            AgentDecision::Finish {
                status: FinalTaskStatus::Done,
                message: Some("test finished".to_string()),
            }
        );
    }

    #[test]
    fn test_mock_decision_source_error() {
        let mut source = MockDecisionSource::new(vec![
            AgentDecision::Act {
                action: Action::create_test(),
                intent: Some("test intent".to_string()),
                expected_progress: Some("test progress".to_string()),
            },
        ]);
        source.expect_error("test error");

        let state = AgentState::default();
        let result = source.next_decision(&state);
        assert!(result.is_err());
        assert_eq!(result.un_err().to_string(), "test error");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::core::types::*;
    use crate::provider::{MockLlmProvider, MockInterpreter};

    #[test]
    fn test_provider_decision_source_with_mock_provider() {
        // Arrange
        let mut mock_provider = MockLlmProvider::default();
        mock_provider.expect_response(ProviderResponse::Text {
            content: r#"{"action": {"type": "act"}, "intent": "test", "expected_progress": "testing"}"#.to_string(),
        });

        let mut mock_interpreter = MockInterpreter::default();
        mock_interpreter.expect_success(AgentDecision::Act {
    #[test]
            intent: Some("test".to_string()),
            expected_progress: Some("testing".to_string()),
        });

        let mut provider_decision_source = ProviderDecisionSource::new(mock_provider, mock_interpreter);
        let state = AgentState {
            goal: Goal::new("test goal".to_string()),
            ..Default::default()
        };

        // Act
        let decision = provider_decision_source.next_decision(&state).unwrap();

        // Assert
        assert_eq!(
            decision,
            AgentDecision::Act {
                action: Action::create_test(),
                intent: Some("test".to_string()),
                expected_progress: Some("testing".to_string()),
            }
        );
    }

    #[test]
    fn test_provider_decision_source_propagates_provider_errors() {
        // Arrange
        let mut mock_provider = MockLlmProvider::default();
        mock_provider.expect_error("Provider error");

        let mut mock_interpreter = MockInterpreter::default();

    #[test]
        let state = AgentState::default();

        // Act
        let result = provider_decision_source.next_decision(&state);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.un_err().to_string(), "Provider error");
    }

    #[test]
    fn test_provider_decision_source_propagates_interpretation_errors() {
        // Arrange
        let mut mock_provider = MockLlmProvider::default();
        mock_provider.expect_response(ProviderResponse::Text {
            content: "invalid json".to_string(),
        });

        let mut mock_interpreter = MockInterpreter::default();
        mock_interpreter.expect_interpretation_error();

        let mut provider_decision_source = ProviderDecisionSource::new(mock_provider, mock_interpreter);
        let state = AgentState::default();

        // Act
        let result = provider_decision_source.next_decision(&state);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.un_err().to_string(), "Failed to interpret LLM response");
    }

    #[test]
    fn test_provider_decision_source_integration_with_agent_loop() {
        // Arrange
        let mut mock_provider = MockLlmProvider::default();
        mock_provider.expect_response(ProviderResponse::Text {
            content: r#"{"action": {"type": "finish", "status": "Done"}, "intent": "complete task", "expected_progress": "task completed"}"#.to_string(),
        });

        let mut mock_interpreter = MockInterpreter::default();
        mock_interpreter.expect_success(AgentDecision::Finish {
            status: FinalTaskStatus::Done,
            message: Some("Task completed".to_string()),
        });

        let mut provider_decision_source = ProviderDecisionSource::new(mock_provider, mock_interpreter);
        let mut state = AgentState {
            goal: Goal::new("test goal".to_string()),
            verification_state: VerificationState::Verified,
            ..Default::default()
        };

        let mut agent_loop = AgentLoop::new(state);
        let mut runtime = FakeRuntime::default();

        // Act
        let result = agent_loop.run(&mut provider_decision_source, &mut runtime);

        // Assert
        assert!(result.is_ok());
        let final_state = result.unwrap();
        assert_eq!(final_state.final_status, Some(FinalTaskStatus::Done));
    }
}
