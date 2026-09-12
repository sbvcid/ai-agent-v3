//! Stage 4: Integration tests for Mock LLM & Fake Runtime test doubles.
//!
//! These tests verify the behavioral contracts from the crate's public API,
//! complementing the unit tests inside `test_doubles.rs`.

use ai_agent_v3::{
    Action, ActionResult, ActionType, AgentDecision, FakeRuntime, FinalTaskStatus, MockLlm,
};

// ---------------------------------------------------------------------------
// MockLlm integration tests
// ---------------------------------------------------------------------------

#[test]
fn mock_llm_scripted_sequence_is_deterministic() {
    // Run the same script twice — results must be identical.
    let script = vec![
        AgentDecision::observe("step 1"),
        AgentDecision::act(Action::new("act-1", ActionType::Execute)),
        AgentDecision::finish(FinalTaskStatus::Done),
    ];

    let run = |decisions: Vec<AgentDecision>| -> Vec<AgentDecision> {
        let mut mock = MockLlm::with_decisions(decisions);
        (0..3).map(|_| mock.next_decision()).collect()
    };

    let run1 = run(script.clone());
    let run2 = run(script);

    assert_eq!(
        run1, run2,
        "Two runs with identical script must produce identical output"
    );
}

#[test]
fn mock_llm_fallback_does_not_panic_after_many_calls() {
    let mut mock = MockLlm::with_decisions(vec![AgentDecision::observe("only one")]);
    // Exhaust the single decision, then call 100 more times.
    for i in 0..101 {
        let d = mock.next_decision();
        if i == 0 {
            assert!(matches!(d, AgentDecision::Observe { .. }));
        } else {
            assert!(matches!(
                d,
                AgentDecision::Finish {
                    status: FinalTaskStatus::Blocked { .. },
                    ..
                }
            ));
        }
    }
    assert_eq!(mock.call_count(), 101);
}

// ---------------------------------------------------------------------------
// FakeRuntime integration tests
// ---------------------------------------------------------------------------

#[test]
fn fake_runtime_mixed_scripted_and_unscripted() {
    let mut rt = FakeRuntime::new();
    rt.add_result("known", ActionResult::success("known", "scripted output"));

    let r1 = rt.execute(Action::new("known", ActionType::Execute));
    let r2 = rt.execute(Action::new("surprise", ActionType::Execute));

    assert!(r1.success);
    assert_eq!(r1.output.as_deref(), Some("scripted output"));

    assert!(!r2.success);
    assert!(r2.error.as_ref().unwrap().contains("no scripted result"));

    // Both actions recorded
    assert_eq!(rt.action_count(), 2);
    assert_eq!(rt.recorded_actions()[0].id, "known");
    assert_eq!(rt.recorded_actions()[1].id, "surprise");
}

#[test]
fn fake_runtime_same_action_id_can_be_called_multiple_times() {
    let mut rt = FakeRuntime::new();
    rt.add_result("repeat", ActionResult::success("repeat", "same result"));

    let r1 = rt.execute(Action::new("repeat", ActionType::Execute));
    let r2 = rt.execute(Action::new("repeat", ActionType::Execute));

    assert_eq!(
        r1, r2,
        "Same action_id must return the same scripted result"
    );
    assert_eq!(rt.action_count(), 2);
}

#[test]
fn fake_runtime_preserves_action_parameters() {
    let mut rt = FakeRuntime::new();
    rt.add_result("act-p", ActionResult::success("act-p", "ok"));

    let action = Action::new("act-p", ActionType::Execute)
        .with_parameter("path", "/some/path")
        .with_parameter("content", "hello world")
        .with_intent("write a test file");

    rt.execute(action);

    let recorded = &rt.recorded_actions()[0];
    assert_eq!(
        recorded.parameters.get("path").map(|s| s.as_str()),
        Some("/some/path")
    );
    assert_eq!(
        recorded.parameters.get("content").map(|s| s.as_str()),
        Some("hello world")
    );
    assert_eq!(recorded.intent.as_deref(), Some("write a test file"));
}

// ---------------------------------------------------------------------------
// Combined MockLlm + FakeRuntime (non-loop, single-step)
// ---------------------------------------------------------------------------

#[test]
fn mock_llm_decision_drives_fake_runtime_single_step() {
    // Demonstrate that a decision from MockLlm can feed into FakeRuntime
    // WITHOUT forming a closed loop. This is a single step, not an agent loop.
    let mut mock = MockLlm::with_decisions(vec![AgentDecision::act(
        Action::new("build", ActionType::Execute)
            .with_parameter("cmd", "cargo build")
            .with_intent("compile the project"),
    )]);

    let mut rt = FakeRuntime::new();
    rt.add_result(
        "build",
        ActionResult::success("build", "compilation succeeded"),
    );

    // Step: get one decision from MockLlm
    let decision = mock.next_decision();

    // If it's an Act, execute it on FakeRuntime
    if let AgentDecision::Act { action, .. } = decision {
        let result = rt.execute(action);
        assert!(result.success);
        assert_eq!(result.output.as_deref(), Some("compilation succeeded"));
    } else {
        panic!("Expected Act decision");
    }

    assert_eq!(mock.call_count(), 1);
    assert_eq!(rt.action_count(), 1);
    assert_eq!(rt.recorded_actions()[0].id, "build");
}
