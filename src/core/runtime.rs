//! Core Runtime abstraction boundary.
//!
//! Defines the common [`Runtime`] capability trait that connects Agent Core
//! to environment execution without coupling Core to any specific runtime implementation.

use crate::core::types::{Action, ActionResult};

/// Common capability contract for all computer runtime implementations.
///
/// A Runtime executes a canonical [`Action`] against the environment and
/// returns an objective [`ActionResult`].
///
/// # Invariants
/// - Runtime must NOT make high-level Agent decisions, plan workflows, or verify user goals.
/// - Runtime receives canonical [`Action`] and returns canonical [`ActionResult`].
/// - No `ToolCall` or `ToolResult` leakage across this boundary.
pub trait Runtime {
    /// Execute an action against the environment and return the structured result.
    fn execute(&mut self, action: Action) -> ActionResult;
}
