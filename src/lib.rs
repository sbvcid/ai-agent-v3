//! AI Agent v3
//!
//! A Rust-based autonomous computer agent.

pub mod core;
pub mod provider;
pub mod runtime;

pub use core::{
    interpret, Action, ActionResult, ActionType, AgentDecision, AgentLoop, AgentState,
    CheckpointError, CheckpointStore, EventTrace, ExecutionState, FakeRuntime, FinalTaskStatus,
    Goal, InterpretationError, JsonFileCheckpointStore, KnowledgeState, LoopError, LoopEvent,
    LoopStepOutcome, MockLlm, Observation, ObservationKind, ProcessSpec, StateCheckpoint,
    ValidationError, VerificationState,
};
pub use provider::{
    ollama::OllamaProvider, LlmProvider, ProviderError, ProviderMessage, ProviderRequest,
    ProviderResponse, ProviderToolCall,
};
