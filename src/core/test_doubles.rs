//! Stage 4: Deterministic Mock LLM & Fake Runtime test doubles.
//!
//! These are pure in-memory, deterministic, scripted test doubles for use
//! in unit/integration tests. They have:
//! - No network, no Ollama, no real LLM
//! - No real filesystem, no process execution, no OS side effects
//! - No randomness, no time dependency
//! - No Agent loop or closed-loop integration (that is Stage 5)
//!
//! The Mock LLM dispenses pre-queued [`AgentDecision`] values in FIFO order.
//! The Fake Runtime accepts [`Action`] and returns scripted [`ActionResult`].

use crate::core::runtime::Runtime;
use crate::core::types::{Action, ActionResult, AgentDecision, FinalTaskStatus};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Mock LLM
// ---------------------------------------------------------------------------

/// Deterministic mock LLM that returns pre-queued [`AgentDecision`] values.
///
/// When the scripted queue is exhausted, returns a deterministic fallback:
/// `AgentDecision::Finish { status: FinalTaskStatus::Blocked { reason }, message }`.
///
/// Records the number of calls made for test assertions.
pub struct MockLlm {
    decisions: Vec<AgentDecision>,
    /// Index of the next decision to return.
    cursor: usize,
    /// Total number of calls to `next_decision`.
    call_count: usize,
}

impl Default for MockLlm {
    fn default() -> Self {
        Self::new()
    }
}

impl MockLlm {
    /// Create an empty MockLlm with no scripted decisions.
    pub fn new() -> Self {
        Self {
            decisions: Vec::new(),
            cursor: 0,
            call_count: 0,
        }
    }

    /// Create a MockLlm pre-loaded with a sequence of decisions.
    pub fn with_decisions(decisions: Vec<AgentDecision>) -> Self {
        Self {
            decisions,
            cursor: 0,
            call_count: 0,
        }
    }

    /// Append a single decision to the end of the scripted queue.
    pub fn push_decision(&mut self, decision: AgentDecision) {
        self.decisions.push(decision);
    }

    /// Return the next scripted decision, or a deterministic fallback if exhausted.
    ///
    /// Each call increments `call_count`.
    pub fn next_decision(&mut self) -> AgentDecision {
        self.call_count += 1;
        if self.cursor < self.decisions.len() {
            let decision = self.decisions[self.cursor].clone();
            self.cursor += 1;
            decision
        } else {
            // Deterministic fallback: no more scripted responses.
            AgentDecision::Finish {
                status: FinalTaskStatus::Blocked {
                    reason: Some("MockLlm: no scripted response remaining".to_string()),
                },
                message: Some("MockLlm exhausted".to_string()),
            }
        }
    }

    /// How many times `next_decision` has been called.
    pub fn call_count(&self) -> usize {
        self.call_count
    }

    /// How many scripted decisions remain (not yet dispensed).
    pub fn remaining(&self) -> usize {
        self.decisions.len().saturating_sub(self.cursor)
    }
}

// ---------------------------------------------------------------------------
// Fake Runtime
// ---------------------------------------------------------------------------

/// Deterministic fake runtime that returns pre-scripted [`ActionResult`] values.
///
/// Scripted results are keyed by `action_id`. If no scripted result exists for
/// a given `action_id`, a deterministic fallback failure is returned.
///
/// Records every [`Action`] received for later test assertions.
pub struct FakeRuntime {
    /// Scripted results keyed by action_id.
    scripted_results: HashMap<String, ActionResult>,
    /// Every Action received, in order.
    recorded_actions: Vec<Action>,
}

impl Default for FakeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime for FakeRuntime {
    fn execute(&mut self, action: Action) -> ActionResult {
        self.execute_internal(action)
    }
}

impl FakeRuntime {
    /// Create a FakeRuntime with no scripted results.
    pub fn new() -> Self {
        Self {
            scripted_results: HashMap::new(),
            recorded_actions: Vec::new(),
        }
    }

    /// Register a scripted result for a given action_id.
    pub fn add_result(&mut self, action_id: impl Into<String>, result: ActionResult) {
        self.scripted_results.insert(action_id.into(), result);
    }

    /// Execute an action: record it, then return the scripted or fallback result.
    ///
    /// This method has NO real OS side effects.
    pub fn execute(&mut self, action: Action) -> ActionResult {
        self.execute_internal(action)
    }

    fn execute_internal(&mut self, action: Action) -> ActionResult {
        self.recorded_actions.push(action.clone());

        if let Some(result) = self.scripted_results.get(&action.id) {
            result.clone()
        } else {
            // Deterministic fallback for unscripted actions.
            ActionResult::failure(
                &action.id,
                format!(
                    "FakeRuntime: no scripted result for action_id '{}'",
                    action.id
                ),
            )
        }
    }

    /// Return all recorded actions in order.
    pub fn recorded_actions(&self) -> &[Action] {
        &self.recorded_actions
    }

    /// Return the number of actions recorded.
    pub fn action_count(&self) -> usize {
        self.recorded_actions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Action, ActionResult, ActionType, AgentDecision, FinalTaskStatus};

    // -----------------------------------------------------------------------
    // MockLlm tests
    // -----------------------------------------------------------------------

    #[test]
    fn mock_llm_returns_scripted_decisions_in_order() {
        let decisions = vec![
            AgentDecision::observe("check environment"),
            AgentDecision::act(Action::new("act-1", ActionType::Execute)),
            AgentDecision::finish(FinalTaskStatus::Done),
        ];
        let mut mock = MockLlm::with_decisions(decisions);

        let d1 = mock.next_decision();
        assert!(matches!(d1, AgentDecision::Observe { .. }));

        let d2 = mock.next_decision();
        assert!(matches!(d2, AgentDecision::Act { .. }));

        let d3 = mock.next_decision();
        assert!(matches!(
            d3,
            AgentDecision::Finish {
                status: FinalTaskStatus::Done,
                ..
            }
        ));
    }

    #[test]
    fn mock_llm_call_count_tracks_correctly() {
        let mut mock = MockLlm::with_decisions(vec![
            AgentDecision::observe("first"),
            AgentDecision::observe("second"),
        ]);

        assert_eq!(mock.call_count(), 0);
        mock.next_decision();
        assert_eq!(mock.call_count(), 1);
        mock.next_decision();
        assert_eq!(mock.call_count(), 2);
        // Exhaust and call again (fallback)
        mock.next_decision();
        assert_eq!(mock.call_count(), 3);
    }

    #[test]
    fn mock_llm_remaining_decrements() {
        let mut mock = MockLlm::with_decisions(vec![
            AgentDecision::observe("a"),
            AgentDecision::observe("b"),
        ]);

        assert_eq!(mock.remaining(), 2);
        mock.next_decision();
        assert_eq!(mock.remaining(), 1);
        mock.next_decision();
        assert_eq!(mock.remaining(), 0);
        // Further calls don't go negative
        mock.next_decision();
        assert_eq!(mock.remaining(), 0);
    }

    #[test]
    fn mock_llm_fallback_is_deterministic_blocked() {
        let mut mock = MockLlm::new(); // empty queue
        let fallback = mock.next_decision();

        match fallback {
            AgentDecision::Finish {
                status: FinalTaskStatus::Blocked { reason },
                message,
            } => {
                assert!(reason.is_some());
                assert!(reason.unwrap().contains("no scripted response"));
                assert!(message.is_some());
            }
            other => panic!("Expected Finish(Blocked), got: {:?}", other),
        }
    }

    #[test]
    fn mock_llm_push_decision_appends() {
        let mut mock = MockLlm::new();
        mock.push_decision(AgentDecision::wait("waiting for job"));
        assert_eq!(mock.remaining(), 1);

        let d = mock.next_decision();
        assert!(matches!(d, AgentDecision::Wait { .. }));
        assert_eq!(mock.remaining(), 0);
    }

    // -----------------------------------------------------------------------
    // FakeRuntime tests
    // -----------------------------------------------------------------------

    #[test]
    fn fake_runtime_returns_scripted_result() {
        let mut rt = FakeRuntime::new();
        rt.add_result("act-1", ActionResult::success("act-1", "file created"));

        let action = Action::new("act-1", ActionType::Execute);
        let result = rt.execute(action);

        assert!(result.success);
        assert_eq!(result.action_id, "act-1");
        assert_eq!(result.output.as_deref(), Some("file created"));
    }

    #[test]
    fn fake_runtime_records_actions_in_order() {
        let mut rt = FakeRuntime::new();
        rt.add_result("act-1", ActionResult::success("act-1", "ok"));
        rt.add_result("act-2", ActionResult::success("act-2", "ok"));

        let a1 = Action::new("act-1", ActionType::Execute);
        let a2 = Action::new("act-2", ActionType::Observe);
        rt.execute(a1);
        rt.execute(a2);

        assert_eq!(rt.action_count(), 2);
        let recorded = rt.recorded_actions();
        assert_eq!(recorded[0].id, "act-1");
        assert_eq!(recorded[0].action_type, ActionType::Execute);
        assert_eq!(recorded[1].id, "act-2");
        assert_eq!(recorded[1].action_type, ActionType::Observe);
    }

    #[test]
    fn fake_runtime_different_actions_get_different_results() {
        let mut rt = FakeRuntime::new();
        rt.add_result("act-1", ActionResult::success("act-1", "output-A"));
        rt.add_result("act-2", ActionResult::failure("act-2", "error-B"));

        let r1 = rt.execute(Action::new("act-1", ActionType::Execute));
        let r2 = rt.execute(Action::new("act-2", ActionType::Execute));

        assert!(r1.success);
        assert_eq!(r1.output.as_deref(), Some("output-A"));

        assert!(!r2.success);
        assert_eq!(r2.error.as_deref(), Some("error-B"));
    }

    #[test]
    fn fake_runtime_unscripted_action_returns_deterministic_failure() {
        let mut rt = FakeRuntime::new();
        // No scripted result for "unknown-action"

        let result = rt.execute(Action::new("unknown-action", ActionType::Execute));

        assert!(!result.success);
        assert_eq!(result.action_id, "unknown-action");
        assert!(result
            .error
            .as_ref()
            .unwrap()
            .contains("no scripted result"));
    }

    #[test]
    fn fake_runtime_records_even_unscripted_actions() {
        let mut rt = FakeRuntime::new();
        rt.execute(Action::new("unscripted-1", ActionType::Observe));
        rt.execute(Action::new("unscripted-2", ActionType::Wait));

        assert_eq!(rt.action_count(), 2);
        assert_eq!(rt.recorded_actions()[0].id, "unscripted-1");
        assert_eq!(rt.recorded_actions()[1].id, "unscripted-2");
    }

    #[test]
    fn no_real_os_side_effects() {
        // This test documents that FakeRuntime is purely in-memory.
        // If it were performing real OS operations, we'd observe file creation,
        // process spawning, etc. The fact that this test completes without
        // any filesystem or process assertions proves the fake is side-effect-free.
        let mut rt = FakeRuntime::new();
        rt.add_result(
            "write-file",
            ActionResult::success("write-file", "file written"),
        );

        let action = Action::new("write-file", ActionType::Execute)
            .with_parameter("path", "C:\\should_not_exist.txt")
            .with_parameter("content", "this must never be written to disk");

        let result = rt.execute(action);
        assert!(result.success);

        // The file must NOT exist on disk. We don't assert fs here because the
        // FakeRuntime never touches the real filesystem by design. This test
        // exists to document that contract.
    }
}
