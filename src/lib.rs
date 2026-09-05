//! AI Agent v3
//!
//! A Rust-based autonomous computer agent.

pub mod core;

pub use core::{
    Action, ActionType, ActionResult, AgentDecision, AgentState, CheckpointError, CheckpointStore,
    ExecutionState, FinalTaskStatus, Goal, JsonFileCheckpointStore, KnowledgeState, Observation,
    ObservationKind, StateCheckpoint, ValidationError, VerificationState,
};
