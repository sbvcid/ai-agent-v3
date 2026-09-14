//! AI Agent v3
//!
//! A Rust-based autonomous computer agent.

pub mod core;
pub mod provider;
pub mod runtime;

pub use core::{
    interpret, Action, ActionResult, ActionType, AgentDecision, AgentLoop, AgentState,
    CheckpointError, CheckpointStore, CompiledContext, ContextCompileError, ContextCompiler,
    DecisionSource, DecisionSourceError, DefaultContextCompiler, EventTrace, ExecutionState,
    FakeLlmProvider, FakeRuntime, FinalTaskStatus, Goal, InMemoryObservationStore,
    InterpretationError, JsonFileCheckpointStore, KnowledgeState, LoopError, LoopEvent,
    LoopStepOutcome, MockLlm, Observation, ObservationKind, ObservationStore,
    ObservationStoreError, ProcessSpec, ProviderDecisionSource, Runtime, StateCheckpoint,
    ValidationError, VerificationState,
};
pub use provider::{
    ollama::OllamaProvider, LlmProvider, ProviderError, ProviderMessage, ProviderRequest,
    ProviderResponse, ProviderToolCall,
};
pub use runtime::process_runtime::ProcessRuntime;
