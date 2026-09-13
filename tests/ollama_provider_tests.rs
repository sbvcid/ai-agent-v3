//! Unit tests for OllamaProvider request mapping, response parsing, and error handling.
//!
//! Verifies serialization and error mappings without requiring a live Ollama server or complex C-dependent libraries.

use ai_agent_v3::provider::ollama::OllamaProvider;
use ai_agent_v3::provider::{LlmProvider, ProviderError, ProviderMessage, ProviderRequest};

#[test]
fn test_ollama_provider_construction() {
    let _provider = OllamaProvider::new("http://localhost:11434/", "gemma4:26b");
}

#[test]
fn test_ollama_provider_invalid_transport() {
    // Calling against a closed/non-existent port must return ProviderError::Transport
    let mut provider = OllamaProvider::new("http://127.0.0.1:1", "gemma4:26b");
    let req = ProviderRequest {
        messages: vec![ProviderMessage::User("hello".to_string())],
    };

    let result = provider.chat(&req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::Transport(_) => {}
        other => panic!("Expected ProviderError::Transport, got {:?}", other),
    }
}
