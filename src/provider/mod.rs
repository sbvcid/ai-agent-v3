//! Provider abstraction layer and Ollama adapter.
//!
//! Defines the provider-neutral [`LlmProvider`] trait and canonical request/response
//! data structures (`ProviderRequest`, `ProviderResponse`, `ProviderMessage`, `ProviderToolCall`, `ProviderError`),
//! and implements [`OllamaProvider`] for Ollama's native `/api/chat` HTTP endpoint.

pub mod ollama;
pub mod decision_source;

use serde::{Deserialize, Serialize};

/// Provider-neutral capability contract for LLM integration.
pub trait LlmProvider {
    /// Send a chat request to the LLM provider and return a structured response.
    fn chat(&mut self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError>;
}

/// Provider-neutral chat completion request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub messages: Vec<ProviderMessage>,
}

/// Provider-neutral chat message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "role", content = "content", rename_all = "snake_case")]
pub enum ProviderMessage {
    System(String),
    User(String),
    Assistant(String),
}

/// Provider-neutral chat completion response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub content: String,
    pub tool_calls: Vec<ProviderToolCall>,
    pub finish_reason: Option<String>,
}

/// Provider-neutral tool call proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Provider-neutral error representation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderError {
    #[error("Transport or network error: {0}")]
    Transport(String),

    #[error("Invalid response or malformed payload: {0}")]
    InvalidResponse(String),

    #[error("Provider rate-limited or unavailable: {0}")]
    Unavailable(String),
}
