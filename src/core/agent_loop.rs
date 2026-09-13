//! Stage 5: Deterministic Closed-Loop Agent & Event Trace.
//!
//! Provides a minimal, deterministic, purely in-memory Agent Loop that orchestrates:
//! ```text
//! AgentState
//!     ↓
//! Mock LLM
//!     ↓
//! AgentDecision
//!     ↓
//! Action
//!     ↓
//! Fake Runtime
//!     ↓
//! ActionResult
//!     ↓
//! Observation
//!     ↓
//! AgentState update
//!     ↓
//! next loop / Finish
//! ```
//!
//! Maintains canonical boundaries:
//! - Uses only canonical `Action`, `ActionResult`, and `Observation`.
//! - No `ToolCall` or `ToolResult`.
//! - Purely in-memory, deterministic, no OS or network side effects.

use crate::core::checkpoint::StateCheckpoint;
use crate::core::runtime::Runtime;
use crate::core::test_doubles::MockLlm;
use crate::core::types::{
    Action, ActionResult, AgentDecision, AgentState, ExecutionState, FinalTaskStatus, Observation,
    ObservationKind, ValidationError, VerificationState,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Loop Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LoopError {
    #[error("Maximum step limit ({0}) exceeded")]
    MaxStepsExceeded(usize),

    #[error("AgentState validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("Cannot step loop: task already finalized with status: {0:?}")]
    AlreadyFinished(FinalTaskStatus),

    #[error(
        "Cannot finish task as Done: Goal has not been verified (verification state is {0:?})"
    )]
    UnverifiedGoal(VerificationState),
}

// ---------------------------------------------------------------------------
// Event Trace
// ---------------------------------------------------------------------------

/// Individual lifecycle event recorded during loop execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopEvent {
    DecisionProduced(AgentDecision),
    ActionExecuted(Action),
    ActionResultReceived(ActionResult),
    ObservationProduced(Observation),
    StateUpdated,
    GoalVerified(String),
    WaitingEntered { reason: Option<String> },
    Finished(FinalTaskStatus),
}

/// Deterministic in-memory event trace of loop execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EventTrace {
    events: Vec<LoopEvent>,
}

impl EventTrace {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: LoopEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[LoopEvent] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

// ---------------------------------------------------------------------------
// Step Outcome
// ---------------------------------------------------------------------------

/// Outcome of a single step in the Agent Loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopStepOutcome {
    /// Step completed; loop should continue with next iteration.
    Continue,
    /// Agent transitioned to Waiting state.
    Waiting,
    /// Agent transitioned to finished with a final task status.
    Finished(FinalTaskStatus),
}

// ---------------------------------------------------------------------------
// Agent Loop
// ---------------------------------------------------------------------------

/// Minimal deterministic Agent Loop.
pub struct AgentLoop {
    state: AgentState,
    trace: EventTrace,
    max_steps: usize,
    current_step: usize,
}

impl AgentLoop {
    /// Create a new AgentLoop with default step limit (50).
    pub fn new(state: AgentState) -> Self {
        Self {
            state,
            trace: EventTrace::new(),
            max_steps: 50,
            current_step: 0,
        }
    }

    /// Create an AgentLoop from an existing StateCheckpoint, restoring state and execution cursor.
    ///
    /// Per `docs/02` §81, Checkpoint preserves AgentState, Goal, Knowledge States, and Event Cursor.
    pub fn from_checkpoint(checkpoint: &StateCheckpoint) -> Self {
        Self {
            state: checkpoint.state.clone(),
            trace: EventTrace::new(),
            max_steps: 50,
            current_step: checkpoint.step_index as usize,
        }
    }

    /// Record verification evidence and mark the goal as Verified.
    ///
    /// Per `docs/02` §51-53, Goal completion requires verification evidence connecting
    /// observations to the original user goal (Action Success != Goal Success).
    pub fn verify_goal(&mut self, evidence: impl Into<String>) {
        let ev = evidence.into();
        self.trace.push(LoopEvent::GoalVerified(ev.clone()));
        self.state.verify_goal(ev);
    }

    /// Configure maximum step limit as a safety boundary.
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Immutable reference to the current working memory.
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// Mutable reference to the working memory (e.g. for inspection or hypothesis revision).
    pub fn state_mut(&mut self) -> &mut AgentState {
        &mut self.state
    }

    /// Immutable reference to the recorded event trace.
    pub fn trace(&self) -> &EventTrace {
        &self.trace
    }

    /// Current step count.
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Execute a single step in the loop.
    pub fn step<R: Runtime>(
        &mut self,
        llm: &mut MockLlm,
        runtime: &mut R,
    ) -> Result<LoopStepOutcome, LoopError> {
        // Prevent stepping if already finalized
        if let Some(status) = &self.state.final_status {
            return Err(LoopError::AlreadyFinished(status.clone()));
        }

        // Enforce max step bound
        if self.current_step >= self.max_steps {
            return Err(LoopError::MaxStepsExceeded(self.max_steps));
        }

        self.current_step += 1;

        // 1. LLM Decision
        let decision = llm.next_decision();
        self.trace
            .push(LoopEvent::DecisionProduced(decision.clone()));

        // 2. Process Decision
        let outcome = match decision {
            AgentDecision::Act {
                action,
                intent,
                expected_progress,
            } => {
                let _ = (intent, expected_progress);
                self.trace.push(LoopEvent::ActionExecuted(action.clone()));

                // 3. Runtime execution
                let action_result = runtime.execute(action.clone());
                self.trace
                    .push(LoopEvent::ActionResultReceived(action_result.clone()));

                // 4. Observation produced
                let obs_id = format!("obs-{}", self.current_step);
                let observation = Observation::from_action_result(obs_id, &action_result);
                self.trace
                    .push(LoopEvent::ObservationProduced(observation.clone()));

                // 5. Update state
                self.state.record_action(action);
                self.state.record_observation(observation);
                self.trace.push(LoopEvent::StateUpdated);

                LoopStepOutcome::Continue
            }

            AgentDecision::Observe { intent } => {
                let obs_id = format!("obs-{}", self.current_step);
                let summary = intent.unwrap_or_else(|| "Observation".to_string());
                let observation = Observation::new(obs_id, ObservationKind::Environment, summary);
                self.trace
                    .push(LoopEvent::ObservationProduced(observation.clone()));

                self.state.record_observation(observation);
                self.trace.push(LoopEvent::StateUpdated);

                LoopStepOutcome::Continue
            }

            AgentDecision::Wait { reason } => {
                self.state.execution_state = ExecutionState::Waiting;
                self.trace.push(LoopEvent::WaitingEntered { reason });
                LoopStepOutcome::Waiting
            }

            AgentDecision::Finish { status, message } => {
                let _ = message;
                // Goal Verification required before Done (docs/02 §51-53: Action Success != Goal Success)
                if status == FinalTaskStatus::Done
                    && self.state.verification_state != VerificationState::Verified
                {
                    return Err(LoopError::UnverifiedGoal(self.state.verification_state));
                }
                self.state.finish(status.clone());
                self.trace.push(LoopEvent::Finished(status.clone()));
                LoopStepOutcome::Finished(status)
            }
        };

        // 6. Validate state invariant after transition
        self.state.validate()?;

        Ok(outcome)
    }

    /// Run the loop to completion (until Finished, Waiting, or error).
    pub fn run<R: Runtime>(
        &mut self,
        llm: &mut MockLlm,
        runtime: &mut R,
    ) -> Result<&AgentState, LoopError> {
        loop {
            match self.step(llm, runtime)? {
                LoopStepOutcome::Continue => continue,
                LoopStepOutcome::Waiting => break,
                LoopStepOutcome::Finished(_) => break,
            }
        }
        Ok(&self.state)
    }
}
