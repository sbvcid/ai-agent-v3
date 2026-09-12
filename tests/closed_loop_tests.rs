//! Stage 5: Deterministic Closed-Loop & Event Trace integration tests.
//!
//! Verifies:
//! - AT-CORE-001: Failure observation -> Strategy revision -> Success -> Finish(Done)
//! - Terminal state variations: Blocked, Impossible, NeedUser, Wait
//! - AgentState invariants: Running + final_status rejected
//! - Deterministic Event Trace equivalence across identical runs
//! - Crash -> Checkpoint -> Resume -> Finish closed-loop continuity
//! - Step count safety bounds

use ai_agent_v3::{
    Action, ActionResult, ActionType, AgentDecision, AgentLoop, AgentState, CheckpointStore,
    ExecutionState, FakeRuntime, FinalTaskStatus, Goal, JsonFileCheckpointStore, LoopError,
    LoopEvent, LoopStepOutcome, MockLlm, StateCheckpoint, ValidationError,
};
use std::fs;

// ---------------------------------------------------------------------------
// Test 1: AT-CORE-001: Failure -> Revision -> Success -> Finish(Done)
// ---------------------------------------------------------------------------

#[test]
fn test_at_core_001_closed_loop_recovery_and_finish() {
    let goal = Goal::new("Build project and fix compilation errors");
    let state = AgentState::new(goal);

    // Mock LLM scripted decisions:
    // Step 1: Attempt action A (cargo check)
    // Step 2: After failure, adapt strategy with action B (fix config)
    // Step 3: Verify and finish
    let decisions = vec![
        AgentDecision::act(
            Action::new("act-cargo-check", ActionType::Execute)
                .with_parameter("cmd", "cargo check")
                .with_intent("Check build status"),
        ),
        AgentDecision::act(
            Action::new("act-fix-config", ActionType::Execute)
                .with_parameter("cmd", "update dependency in Cargo.toml")
                .with_intent("Revise hypothesis and fix dependency version"),
        ),
        AgentDecision::finish(FinalTaskStatus::Done),
    ];
    let mut mock_llm = MockLlm::with_decisions(decisions);

    // Fake Runtime scripted results:
    // act-cargo-check -> failure
    // act-fix-config -> success
    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-cargo-check",
        ActionResult::failure("act-cargo-check", "mismatched types in crate foo"),
    );
    runtime.add_result(
        "act-fix-config",
        ActionResult::success("act-fix-config", "dependency updated successfully"),
    );

    let mut agent_loop = AgentLoop::new(state);

    // Execute closed loop to completion
    let final_state = agent_loop
        .run(&mut mock_llm, &mut runtime)
        .expect("AgentLoop must complete without error");

    // 1. Terminal task status must be Done
    assert_eq!(final_state.final_status, Some(FinalTaskStatus::Done));
    assert_eq!(final_state.execution_state, ExecutionState::Waiting);

    // 2. Both actions were recorded in working memory in order
    assert_eq!(final_state.recent_actions.len(), 2);
    assert_eq!(final_state.recent_actions[0].id, "act-cargo-check");
    assert_eq!(final_state.recent_actions[1].id, "act-fix-config");

    // 3. Observations were recorded connecting environment to agent
    assert_eq!(final_state.recent_observations.len(), 2);
    assert_eq!(final_state.recent_observations[0].id, "obs-1");
    assert!(final_state.recent_observations[0]
        .summary
        .contains("mismatched types"));
    assert_eq!(final_state.recent_observations[1].id, "obs-2");
    assert!(final_state.recent_observations[1]
        .summary
        .contains("dependency updated"));

    // 4. Runtime executed exactly 2 actions with zero OS side effects
    assert_eq!(runtime.action_count(), 2);
    assert_eq!(runtime.recorded_actions()[0].id, "act-cargo-check");
    assert_eq!(runtime.recorded_actions()[1].id, "act-fix-config");

    // 5. Mock LLM decisions fully consumed (no exhaustion fallback)
    assert_eq!(mock_llm.remaining(), 0);
    assert_eq!(mock_llm.call_count(), 3);

    // 6. Verify Event Trace sequence
    let events = agent_loop.trace().events();
    assert_eq!(events.len(), 12);
    assert!(matches!(
        events[0],
        LoopEvent::DecisionProduced(AgentDecision::Act { .. })
    ));
    assert!(matches!(events[1], LoopEvent::ActionExecuted(_)));
    assert!(matches!(events[2], LoopEvent::ActionResultReceived(ref r) if !r.success));
    assert!(matches!(events[3], LoopEvent::ObservationProduced(_)));
    assert!(matches!(events[4], LoopEvent::StateUpdated));
    assert!(matches!(
        events[5],
        LoopEvent::DecisionProduced(AgentDecision::Act { .. })
    ));
    assert!(matches!(events[6], LoopEvent::ActionExecuted(_)));
    assert!(matches!(events[7], LoopEvent::ActionResultReceived(ref r) if r.success));
    assert!(matches!(events[8], LoopEvent::ObservationProduced(_)));
    assert!(matches!(events[9], LoopEvent::StateUpdated));
    assert!(matches!(
        events[10],
        LoopEvent::DecisionProduced(AgentDecision::Finish { .. })
    ));
    assert!(matches!(
        events[11],
        LoopEvent::Finished(FinalTaskStatus::Done)
    ));
}

// ---------------------------------------------------------------------------
// Test 2: Failure -> Blocked
// ---------------------------------------------------------------------------

#[test]
fn test_closed_loop_failure_leads_to_blocked() {
    let goal = Goal::new("Access protected resource");
    let state = AgentState::new(goal);

    let decisions = vec![
        AgentDecision::act(Action::new("act-auth", ActionType::Execute)),
        AgentDecision::finish(FinalTaskStatus::Blocked {
            reason: Some("Access denied by policy".to_string()),
        }),
    ];
    let mut mock_llm = MockLlm::with_decisions(decisions);

    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-auth",
        ActionResult::failure("act-auth", "403 Forbidden"),
    );

    let mut agent_loop = AgentLoop::new(state);
    let final_state = agent_loop
        .run(&mut mock_llm, &mut runtime)
        .expect("Must run");

    assert_eq!(
        final_state.final_status,
        Some(FinalTaskStatus::Blocked {
            reason: Some("Access denied by policy".to_string())
        })
    );
    assert_eq!(final_state.execution_state, ExecutionState::Waiting);
    assert_eq!(mock_llm.remaining(), 0);
}

// ---------------------------------------------------------------------------
// Test 3: Terminal variants: Impossible & NeedUser
// ---------------------------------------------------------------------------

#[test]
fn test_closed_loop_terminal_variants() {
    for status in [
        FinalTaskStatus::Impossible {
            reason: Some("No compiler exists".to_string()),
        },
        FinalTaskStatus::NeedUser {
            reason: Some("2FA code required".to_string()),
        },
    ] {
        let goal = Goal::new("Test terminal variant");
        let state = AgentState::new(goal);
        let mut mock = MockLlm::with_decisions(vec![AgentDecision::finish(status.clone())]);
        let mut rt = FakeRuntime::new();

        let mut agent_loop = AgentLoop::new(state);
        let final_state = agent_loop.run(&mut mock, &mut rt).expect("Must finish");

        assert_eq!(final_state.final_status, Some(status));
        assert_eq!(final_state.execution_state, ExecutionState::Waiting);
        assert_eq!(mock.remaining(), 0);
    }
}

// ---------------------------------------------------------------------------
// Test 4: Wait decision transitions to Waiting without final_status
// ---------------------------------------------------------------------------

#[test]
fn test_closed_loop_wait_decision() {
    let goal = Goal::new("Compile large codebase");
    let state = AgentState::new(goal);

    let decisions = vec![AgentDecision::wait("Waiting for compilation to finish")];
    let mut mock_llm = MockLlm::with_decisions(decisions);
    let mut runtime = FakeRuntime::new();

    let mut agent_loop = AgentLoop::new(state);
    let outcome = agent_loop
        .step(&mut mock_llm, &mut runtime)
        .expect("Step should succeed");

    assert_eq!(outcome, LoopStepOutcome::Waiting);
    assert_eq!(agent_loop.state().execution_state, ExecutionState::Waiting);
    assert_eq!(agent_loop.state().final_status, None);
    assert_eq!(mock_llm.remaining(), 0);
}

// ---------------------------------------------------------------------------
// Test 5: Invariant verification: Running + final_status = Some rejected
// ---------------------------------------------------------------------------

#[test]
fn test_agent_state_invariant_enforcement_in_loop() {
    let goal = Goal::new("Test invariant in loop");
    let mut state = AgentState::new(goal);

    // Directly violating invariant
    state.execution_state = ExecutionState::Running;
    state.final_status = Some(FinalTaskStatus::Done);

    let err = state.validate();
    assert!(matches!(err, Err(ValidationError::InvariantViolation(_))));

    // Stepping an already finalized loop must be rejected
    let mut valid_finished_state = AgentState::new(Goal::new("Already done"));
    valid_finished_state.finish(FinalTaskStatus::Done);

    let mut agent_loop = AgentLoop::new(valid_finished_state);
    let mut mock = MockLlm::new();
    let mut rt = FakeRuntime::new();

    let step_err = agent_loop.step(&mut mock, &mut rt);
    assert!(matches!(
        step_err,
        Err(LoopError::AlreadyFinished(FinalTaskStatus::Done))
    ));
}

// ---------------------------------------------------------------------------
// Test 6: Determinism: EventTrace and AgentState identical across two runs
// ---------------------------------------------------------------------------

#[test]
fn test_closed_loop_event_trace_determinism() {
    let make_script = || {
        vec![
            AgentDecision::observe("check disk"),
            AgentDecision::act(
                Action::new("run-1", ActionType::Execute).with_parameter("cmd", "dir"),
            ),
            AgentDecision::act(
                Action::new("run-2", ActionType::Execute).with_parameter("cmd", "echo ok"),
            ),
            AgentDecision::finish(FinalTaskStatus::Done),
        ]
    };

    let make_runtime = || {
        let mut rt = FakeRuntime::new();
        rt.add_result("run-1", ActionResult::failure("run-1", "not found"));
        rt.add_result("run-2", ActionResult::success("run-2", "ok"));
        rt
    };

    // Run 1
    let state1 = AgentState::new(Goal::new("Deterministic test"));
    let mut mock1 = MockLlm::with_decisions(make_script());
    let mut rt1 = make_runtime();
    let mut loop1 = AgentLoop::new(state1);
    loop1.run(&mut mock1, &mut rt1).expect("Run 1 failed");

    // Run 2
    let state2 = AgentState::new(Goal::new("Deterministic test"));
    let mut mock2 = MockLlm::with_decisions(make_script());
    let mut rt2 = make_runtime();
    let mut loop2 = AgentLoop::new(state2);
    loop2.run(&mut mock2, &mut rt2).expect("Run 2 failed");

    // Assert absolute equivalence
    assert_eq!(
        loop1.trace(),
        loop2.trace(),
        "Event traces from identical runs must be identical"
    );
    assert_eq!(
        loop1.state(),
        loop2.state(),
        "Agent states from identical runs must be identical"
    );
}

// ---------------------------------------------------------------------------
// Test 7: Crash -> Checkpoint -> Resume -> Finish closed-loop continuity
// ---------------------------------------------------------------------------

#[test]
fn test_closed_loop_crash_and_resume_via_checkpoint() {
    let test_dir =
        std::env::temp_dir().join(format!("test_closed_loop_crash_{}", std::process::id()));
    let _ = fs::remove_dir_all(&test_dir);
    let store = JsonFileCheckpointStore::new(&test_dir).expect("Create store");

    // Context 1: Start task, execute Step 1, then crash
    {
        let goal = Goal::new("Multi-step task with crash recovery");
        let state = AgentState::new(goal);

        let mut mock = MockLlm::with_decisions(vec![AgentDecision::act(
            Action::new("step-1-act", ActionType::Execute)
                .with_parameter("target", "init")
                .with_intent("Initialize component"),
        )]);
        let mut runtime = FakeRuntime::new();
        runtime.add_result(
            "step-1-act",
            ActionResult::success("step-1-act", "initialized"),
        );

        let mut agent_loop = AgentLoop::new(state);
        let outcome = agent_loop
            .step(&mut mock, &mut runtime)
            .expect("Step 1 must succeed");
        assert_eq!(outcome, LoopStepOutcome::Continue);

        // Save checkpoint before crash
        let ckpt = StateCheckpoint::new("ckpt-step-1", 1, agent_loop.state().clone());
        store.save_checkpoint(&ckpt).expect("Save checkpoint");

        // Context 1 drops here (simulated process crash)
    }

    // Context 2: Resume from latest checkpoint and complete remaining work
    {
        let latest = store
            .load_latest_checkpoint()
            .expect("Load checkpoint")
            .expect("Must exist");
        assert_eq!(latest.step_index, 1);
        assert_eq!(latest.state.recent_actions.len(), 1);
        assert_eq!(latest.state.recent_actions[0].id, "step-1-act");
        assert_eq!(latest.state.execution_state, ExecutionState::Running);
        assert_eq!(latest.state.final_status, None);

        // Create new AgentLoop from restored state
        let mut resumed_loop = AgentLoop::new(latest.state);

        // Script remaining work: Step 2 -> Finish
        let remaining_decisions = vec![
            AgentDecision::act(
                Action::new("step-2-act", ActionType::Execute)
                    .with_parameter("target", "deploy")
                    .with_intent("Deploy component"),
            ),
            AgentDecision::finish(FinalTaskStatus::Done),
        ];
        let mut mock = MockLlm::with_decisions(remaining_decisions);

        let mut runtime = FakeRuntime::new();
        runtime.add_result(
            "step-2-act",
            ActionResult::success("step-2-act", "deployed successfully"),
        );

        let final_state = resumed_loop
            .run(&mut mock, &mut runtime)
            .expect("Resumed loop must finish");

        assert_eq!(final_state.final_status, Some(FinalTaskStatus::Done));
        assert_eq!(final_state.execution_state, ExecutionState::Waiting);
        // Total actions across both sessions: 2
        assert_eq!(final_state.recent_actions.len(), 2);
        assert_eq!(final_state.recent_actions[0].id, "step-1-act");
        assert_eq!(final_state.recent_actions[1].id, "step-2-act");
        assert_eq!(mock.remaining(), 0);

        // Save final checkpoint
        let final_ckpt = StateCheckpoint::new("ckpt-step-2", 2, final_state.clone());
        store.save_checkpoint(&final_ckpt).expect("Save final ckpt");
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

// ---------------------------------------------------------------------------
// Test 8: Max steps safety bound
// ---------------------------------------------------------------------------

#[test]
fn test_closed_loop_max_steps_safety_bound() {
    let goal = Goal::new("Infinite action loop test");
    let state = AgentState::new(goal);

    // Provide 5 decisions, but limit loop to 2 steps
    let decisions = vec![
        AgentDecision::act(Action::new("act-1", ActionType::Execute)),
        AgentDecision::act(Action::new("act-2", ActionType::Execute)),
        AgentDecision::act(Action::new("act-3", ActionType::Execute)),
    ];
    let mut mock = MockLlm::with_decisions(decisions);
    let mut runtime = FakeRuntime::new();
    runtime.add_result("act-1", ActionResult::success("act-1", "ok"));
    runtime.add_result("act-2", ActionResult::success("act-2", "ok"));
    runtime.add_result("act-3", ActionResult::success("act-3", "ok"));

    let mut agent_loop = AgentLoop::new(state).with_max_steps(2);

    let err = agent_loop.run(&mut mock, &mut runtime);
    assert_eq!(err, Err(LoopError::MaxStepsExceeded(2)));
    assert_eq!(agent_loop.current_step(), 2);
    assert_eq!(agent_loop.state().recent_actions.len(), 2);
}
