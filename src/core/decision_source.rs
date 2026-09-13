use crate::core::types::{AgentDecision, AgentState, ValidationError};
use thiserror::Error;

/// Abstract source of AgentDecisions for the Agent Loop.
///
/// This trait decouples the Agent Loop from specific LLM implementations,
/// allowing it to work with both deterministic test doubles (MockLlm) and
/// real LLM providers (Ollama, etc.) through a unified interface.
pub trait DecisionSource {
    /// Get the next agent decision based on current state.
    ///
    /// Returns an AgentDecision or an error if decision generation fails.
    fn next_decision(&mut self, state: &AgentState) -> Result<AgentDecision, DecisionSourceError>;
}

/// Errors that can occur when getting decisions from a DecisionSource.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecisionSourceError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Failed to generate decision from source: {0}")]
    GenerationFailed(String),
}