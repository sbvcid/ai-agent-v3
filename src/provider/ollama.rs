//! Ollama provider adapter.
//!
//! Implements [`LlmProvider`] for Ollama's native `/api/chat` HTTP endpoint.
//! Translates provider-neutral [`ProviderRequest`] into Ollama chat request payloads,
//! executes synchronous HTTP requests via `ureq`, and maps Ollama responses back
//! into provider-neutral [`ProviderResponse`] and [`ProviderError`].
//!
//! Ollama-specific configuration and wire protocol details are strictly encapsulated here
//! and never leak into Agent Core or provider-neutral types.

use crate::provider::{
    LlmProvider, ProviderError, ProviderMessage, ProviderRequest, ProviderResponse,
    ProviderToolCall,
};
use serde::{Deserialize, Serialize};

/// Concrete Ollama LLM provider adapter.
pub struct OllamaProvider {
    base_url: String,
    model: String,
}

impl OllamaProvider {
    /// Create a new OllamaProvider with target base URL and model name.
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Ollama Wire Types (Internal to adapter)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    #[serde(default)]
    message: Option<OllamaResponseMessage>,
    #[serde(default)]
    done_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Vec<OllamaToolCall>,
}

#[derive(Debug, Deserialize)]
struct OllamaToolCall {
    #[serde(default)]
    function: Option<OllamaFunctionCall>,
}

#[derive(Debug, Deserialize)]
struct OllamaFunctionCall {
    #[serde(default)]
    name: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

// ---------------------------------------------------------------------------
// LlmProvider Implementation for Ollama
// ---------------------------------------------------------------------------

impl LlmProvider for OllamaProvider {
    fn chat(&mut self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        let endpoint = format!("{}/api/chat", self.base_url);

        let ollama_messages: Vec<OllamaMessage> = request
            .messages
            .iter()
            .map(|msg| match msg {
                ProviderMessage::System(text) => OllamaMessage {
                    role: "system".to_string(),
                    content: text.clone(),
                },
                ProviderMessage::User(text) => OllamaMessage {
                    role: "user".to_string(),
                    content: text.clone(),
                },
                ProviderMessage::Assistant(text) => OllamaMessage {
                    role: "assistant".to_string(),
                    content: text.clone(),
                },
            })
            .collect();

        let payload = OllamaChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: false,
        };

        let response = ureq::post(&endpoint)
            .send_json(&payload)
            .map_err(|e| match e {
                ureq::Error::Status(code, _) => {
                    if code == 429 || code >= 500 {
                        ProviderError::Unavailable(format!("Ollama HTTP status {}: {}", code, e))
                    } else {
                        ProviderError::InvalidResponse(format!(
                            "Ollama HTTP status {}: {}",
                            code, e
                        ))
                    }
                }
                _ => ProviderError::Transport(format!(
                    "Transport error communicating with Ollama: {}",
                    e
                )),
            })?;

        let chat_resp: OllamaChatResponse = response.into_json().map_err(|e| {
            ProviderError::InvalidResponse(format!("Failed to parse Ollama JSON response: {}", e))
        })?;

        let msg = chat_resp.message.unwrap_or(OllamaResponseMessage {
            content: String::new(),
            tool_calls: Vec::new(),
        });

        let mut tool_calls = Vec::new();
        for (idx, tc) in msg.tool_calls.into_iter().enumerate() {
            if let Some(func) = tc.function {
                if !func.name.is_empty() {
                    // Generate deterministic adapter-local ID since Ollama wire format does not supply one
                    let id = format!("ollama-tool-call-{}", idx + 1);
                    tool_calls.push(ProviderToolCall {
                        id,
                        name: func.name,
                        arguments: func.arguments,
                    });
                }
            }
        }

        Ok(ProviderResponse {
            content: msg.content,
            tool_calls,
            finish_reason: chat_resp.done_reason,
        })
    }
}
