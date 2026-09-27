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
    ContextCompiler, DefaultContextCompiler, EvidenceLink, EvidenceRelation, ExecutionState,
    FakeLlmProvider, FakeRuntime, FinalTaskStatus, Goal, InMemoryKnowledgeStore,
    InMemoryObservationStore, JsonFileCheckpointStore, KnowledgeClaim, KnowledgeClaimStatus,
    KnowledgeStore, KnowledgeStoreError, KnowledgeStoreSemanticUpdater, LoopError, LoopEvent,
    LoopStepOutcome, MockLlm, Observation, ObservationKind, ObservationStore,
    ProviderDecisionSource, ProviderMessage, ProviderResponse, ProviderToolCall, SemanticUpdate,
    SemanticUpdateError, SemanticUpdateProducer, StateCheckpoint, Unknown, ValidationError,
    VerificationState,
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

    // Step 1: Action A fails
    let outcome = agent_loop
        .step(&mut mock_llm, &mut runtime)
        .expect("Action A step must succeed");
    assert_eq!(outcome, LoopStepOutcome::Continue);

    // Step 2: Action B succeeds — Action Success != Goal Success
    let outcome = agent_loop
        .step(&mut mock_llm, &mut runtime)
        .expect("Action B step must succeed");
    assert_eq!(outcome, LoopStepOutcome::Continue);
    assert_eq!(
        agent_loop.state().verification_state,
        VerificationState::NotVerified
    );

    // Goal Verification using existing Observation / ActionResult as evidence
    let success_obs = agent_loop
        .state()
        .recent_observations
        .last()
        .expect("Action B must produce an observation");
    let evidence = format!(
        "Original goal '{}': observation {} from action {:?} reports {}",
        agent_loop.state().goal.description,
        success_obs.id,
        success_obs.source_action_id,
        success_obs.summary
    );
    let _ = success_obs;
    agent_loop.verify_goal(evidence);
    assert_eq!(
        agent_loop.state().verification_state,
        VerificationState::Verified
    );

    // Step 3: Finish(Done) only after VerificationState = Verified
    let outcome = agent_loop
        .step(&mut mock_llm, &mut runtime)
        .expect("Finish step must succeed");
    assert_eq!(outcome, LoopStepOutcome::Finished(FinalTaskStatus::Done));

    let final_state = agent_loop.state();

    // 1. Terminal task status must be Done
    assert_eq!(final_state.final_status, Some(FinalTaskStatus::Done));
    assert_eq!(final_state.execution_state, ExecutionState::Waiting);
    assert_eq!(final_state.verification_state, VerificationState::Verified);

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

    // 6. Verify Event Trace sequence includes explicit Goal Verification
    let events = agent_loop.trace().events();
    assert_eq!(events.len(), 13);
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
    assert!(matches!(events[10], LoopEvent::GoalVerified(_)));
    assert!(matches!(
        events[11],
        LoopEvent::DecisionProduced(AgentDecision::Finish { .. })
    ));
    assert!(matches!(
        events[12],
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
    valid_finished_state.verify_goal("Verified: already-done fixture");
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

    fn run_until_verified_done(
        agent_loop: &mut AgentLoop,
        mock: &mut MockLlm,
        rt: &mut FakeRuntime,
    ) {
        // observe + two acts
        for _ in 0..3 {
            agent_loop.step(mock, rt).expect("scripted step");
        }
        assert_eq!(
            agent_loop.state().verification_state,
            VerificationState::NotVerified
        );
        agent_loop.verify_goal("Verified: deterministic evidence from observations");
        agent_loop.run(mock, rt).expect("Finish after verification");
    }

    // Run 1
    let state1 = AgentState::new(Goal::new("Deterministic test"));
    let mut mock1 = MockLlm::with_decisions(make_script());
    let mut rt1 = make_runtime();
    let mut loop1 = AgentLoop::new(state1);
    run_until_verified_done(&mut loop1, &mut mock1, &mut rt1);

    // Run 2
    let state2 = AgentState::new(Goal::new("Deterministic test"));
    let mut mock2 = MockLlm::with_decisions(make_script());
    let mut rt2 = make_runtime();
    let mut loop2 = AgentLoop::new(state2);
    run_until_verified_done(&mut loop2, &mut mock2, &mut rt2);

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
        assert_eq!(agent_loop.current_step(), 1);
        assert_eq!(agent_loop.state().recent_observations[0].id, "obs-1");

        // Save checkpoint before crash, including execution cursor
        let ckpt = StateCheckpoint::new(
            "ckpt-step-1",
            agent_loop.current_step() as u64,
            agent_loop.state().clone(),
        );
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
        assert_eq!(latest.state.recent_observations[0].id, "obs-1");
        assert_eq!(latest.state.execution_state, ExecutionState::Running);
        assert_eq!(latest.state.final_status, None);

        // Restore state AND execution cursor
        let mut resumed_loop = AgentLoop::from_checkpoint(&latest);
        assert_eq!(resumed_loop.current_step(), latest.step_index as usize);
        assert_eq!(resumed_loop.current_step(), 1);

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

        let outcome = resumed_loop
            .step(&mut mock, &mut runtime)
            .expect("Resumed action step must succeed");
        assert_eq!(outcome, LoopStepOutcome::Continue);
        assert_eq!(resumed_loop.current_step(), 2);

        let resumed_obs_ids: Vec<&str> = resumed_loop
            .state()
            .recent_observations
            .iter()
            .map(|o| o.id.as_str())
            .collect();
        assert_eq!(resumed_obs_ids, vec!["obs-1", "obs-2"]);
        assert_ne!(
            resumed_loop.state().recent_observations[1].id,
            resumed_loop.state().recent_observations[0].id,
            "post-resume Observation IDs must not reuse crash-before IDs"
        );

        // Action success after resume is still not Goal Success
        assert_eq!(
            resumed_loop.state().verification_state,
            VerificationState::NotVerified
        );
        let evidence_obs = resumed_loop
            .state()
            .recent_observations
            .last()
            .expect("resume observation")
            .clone();
        let goal_desc = resumed_loop.state().goal.description.clone();
        resumed_loop.verify_goal(format!(
            "Original goal '{}': observation {} reports {}",
            goal_desc, evidence_obs.id, evidence_obs.summary
        ));

        let final_state = resumed_loop
            .run(&mut mock, &mut runtime)
            .expect("Resumed loop must finish")
            .clone();

        assert_eq!(final_state.final_status, Some(FinalTaskStatus::Done));
        assert_eq!(final_state.execution_state, ExecutionState::Waiting);
        assert_eq!(final_state.verification_state, VerificationState::Verified);
        // Total actions across both sessions: 2
        assert_eq!(final_state.recent_actions.len(), 2);
        assert_eq!(final_state.recent_actions[0].id, "step-1-act");
        assert_eq!(final_state.recent_actions[1].id, "step-2-act");
        assert_eq!(mock.remaining(), 0);

        // Save final checkpoint — cursor read after run() completes (borrow fully released)
        let final_ckpt = StateCheckpoint::new(
            "ckpt-step-2",
            resumed_loop.current_step() as u64,
            final_state.clone(),
        );
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

// ---------------------------------------------------------------------------
// Test 9: Action Success is not Goal Success — Finish(Done) rejected
// ---------------------------------------------------------------------------

#[test]
fn test_action_success_without_goal_verification_cannot_finish_done() {
    let goal = Goal::new("Create output.txt with expected checksum");
    let state = AgentState::new(goal);

    let decisions = vec![
        AgentDecision::act(Action::new("act-ok", ActionType::Execute)),
        AgentDecision::finish(FinalTaskStatus::Done),
    ];
    let mut mock = MockLlm::with_decisions(decisions);
    let mut runtime = FakeRuntime::new();
    runtime.add_result("act-ok", ActionResult::success("act-ok", "exit 0"));

    let mut agent_loop = AgentLoop::new(state);
    agent_loop
        .step(&mut mock, &mut runtime)
        .expect("Action success is a valid loop step");

    assert_eq!(
        agent_loop.state().verification_state,
        VerificationState::NotVerified
    );
    assert_eq!(agent_loop.state().recent_actions.len(), 1);

    let err = agent_loop
        .step(&mut mock, &mut runtime)
        .expect_err("Done requires Goal Verification");
    assert!(matches!(
        err,
        LoopError::UnverifiedGoal(VerificationState::NotVerified)
    ));
    assert_eq!(agent_loop.state().final_status, None);
    assert_eq!(agent_loop.state().execution_state, ExecutionState::Running);
}

// ---------------------------------------------------------------------------
// Test 10: Sufficient Goal Verification Evidence allows Done
// ---------------------------------------------------------------------------

#[test]
fn test_goal_verification_evidence_allows_done() {
    let goal = Goal::new("Create output.txt with expected checksum");
    let state = AgentState::new(goal);

    let decisions = vec![
        AgentDecision::act(Action::new("act-ok", ActionType::Execute)),
        AgentDecision::finish(FinalTaskStatus::Done),
    ];
    let mut mock = MockLlm::with_decisions(decisions);
    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-ok",
        ActionResult::success("act-ok", "output.txt exists checksum=abc"),
    );

    let mut agent_loop = AgentLoop::new(state);
    agent_loop
        .step(&mut mock, &mut runtime)
        .expect("Action success");

    let obs = agent_loop.state().recent_observations[0].clone();
    agent_loop.verify_goal(format!(
        "Goal evidence from observation {}: {}",
        obs.id, obs.summary
    ));
    assert_eq!(
        agent_loop.state().verification_state,
        VerificationState::Verified
    );
    assert!(!agent_loop.state().evidence.is_empty());

    let outcome = agent_loop.step(&mut mock, &mut runtime).expect("Done");
    assert_eq!(outcome, LoopStepOutcome::Finished(FinalTaskStatus::Done));
    assert_eq!(agent_loop.state().final_status, Some(FinalTaskStatus::Done));
    assert_eq!(agent_loop.state().execution_state, ExecutionState::Waiting);
}

// ---------------------------------------------------------------------------
// Test 11: Non-Done terminals do not require Goal Verification
// ---------------------------------------------------------------------------

#[test]
fn test_non_done_terminals_do_not_require_goal_verification() {
    for status in [
        FinalTaskStatus::Blocked {
            reason: Some("policy".to_string()),
        },
        FinalTaskStatus::Impossible {
            reason: Some("no compiler".to_string()),
        },
        FinalTaskStatus::NeedUser {
            reason: Some("2FA".to_string()),
        },
    ] {
        let mut mock = MockLlm::with_decisions(vec![AgentDecision::finish(status.clone())]);
        let mut rt = FakeRuntime::new();
        let mut agent_loop = AgentLoop::new(AgentState::new(Goal::new("terminal without verify")));
        assert_eq!(
            agent_loop.state().verification_state,
            VerificationState::NotVerified
        );
        let final_state = agent_loop.run(&mut mock, &mut rt).expect("Must finish");
        assert_eq!(final_state.final_status, Some(status));
        assert_eq!(final_state.execution_state, ExecutionState::Waiting);
        assert_eq!(
            final_state.verification_state,
            VerificationState::NotVerified
        );
    }
}

// ---------------------------------------------------------------------------
// B2-C3: Deterministic Closed-Loop Semantic Integration Tests
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ActionToClaimProducer;

impl SemanticUpdateProducer for ActionToClaimProducer {
    fn produce(&mut self, observation: &Observation) -> Option<SemanticUpdate> {
        let obs_id = observation.id.clone();
        let action_id = observation
            .source_action_id
            .clone()
            .unwrap_or_else(|| "unknown-action".to_string());
        let claim_id = format!("claim-for-{}", obs_id);
        Some(SemanticUpdate::ClaimWithEvidence {
            claim: KnowledgeClaim {
                id: claim_id.clone(),
                subject: action_id,
                predicate: "execution_result".to_string(),
                value: observation.summary.clone(),
                status: KnowledgeClaimStatus::Observed,
                scope: "closed-loop-integration".to_string(),
                evidence_refs: vec![obs_id.clone()],
            },
            evidence: vec![EvidenceLink {
                observation_id: obs_id,
                claim_id,
                relation: EvidenceRelation::Supports,
            }],
        })
    }
}

// ---------------------------------------------------------------------------
// Test 12: Success Action Observation -> Knowledge -> Next ProviderRequest
// ---------------------------------------------------------------------------

#[test]
fn test_b2_c3_closed_loop_success_observation_to_next_provider_request() {
    let goal = Goal::new("Validate semantic closed loop with success action");
    let state = AgentState::new(goal);

    let response_1 = ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: "act-run-check".to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "cargo", "args": ["check"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    };
    let response_2 = ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: "act-run-test".to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "cargo", "args": ["test"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    };

    let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response_1), Ok(response_2)]);
    let mut decision_source = ProviderDecisionSource::new(fake_provider);

    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-run-check",
        ActionResult::success("act-run-check", "build clean: 0 warnings"),
    );
    runtime.add_result(
        "act-run-test",
        ActionResult::success("act-run-test", "test suite passed"),
    );

    let mut agent_loop = AgentLoop::new(state)
        .with_semantic_update_pipeline(ActionToClaimProducer, KnowledgeStoreSemanticUpdater::new());

    // Step 1: Decision source called -> ProviderRequest 1 has no knowledge claims yet
    let outcome_1 = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 1 must succeed");
    assert_eq!(outcome_1, LoopStepOutcome::Continue);

    // Assert step 1 results:
    // 1. Observation was produced and stored authoritatively
    let obs_1 = agent_loop
        .observation_store()
        .get("obs-1")
        .expect("obs-1 must exist in authoritative store");
    assert_eq!(obs_1.source_action_id.as_deref(), Some("act-run-check"));
    assert_eq!(obs_1.summary, "build clean: 0 warnings");

    // 2. KnowledgeClaim and EvidenceLink exist in authoritative KnowledgeStore
    assert_eq!(agent_loop.knowledge_store().claims().len(), 1);
    let claim_1 = &agent_loop.knowledge_store().claims()[0];
    assert_eq!(claim_1.id, "claim-for-obs-1");
    assert_eq!(claim_1.subject, "act-run-check");
    assert_eq!(claim_1.value, "build clean: 0 warnings");
    assert_eq!(claim_1.status, KnowledgeClaimStatus::Observed);
    assert_eq!(claim_1.evidence_refs, vec!["obs-1"]);

    assert_eq!(agent_loop.knowledge_store().evidence().len(), 1);
    let link_1 = &agent_loop.knowledge_store().evidence()[0];
    assert_eq!(link_1.observation_id, "obs-1");
    assert_eq!(link_1.claim_id, "claim-for-obs-1");
    assert_eq!(link_1.relation, EvidenceRelation::Supports);

    // 3. ProviderRequest 1 had empty knowledge context
    let recorded_requests = decision_source.provider().recorded_requests();
    assert_eq!(recorded_requests.len(), 1);
    assert!(recorded_requests[0].knowledge_claims.is_empty());
    assert!(recorded_requests[0].evidence_links.is_empty());

    // Step 2: Next step executes -> ContextCompiler compiles KnowledgeStore -> ProviderRequest 2
    let outcome_2 = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 2 must succeed");
    assert_eq!(outcome_2, LoopStepOutcome::Continue);

    // Assert step 2 closed-loop propagation:
    let recorded_requests = decision_source.provider().recorded_requests();
    assert_eq!(recorded_requests.len(), 2);
    let req_2 = &recorded_requests[1];

    // Provenance link: EvidenceLink in ProviderRequest accurately references obs-1 from previous Action
    assert_eq!(req_2.knowledge_claims.len(), 1);
    assert_eq!(req_2.knowledge_claims[0].id, "claim-for-obs-1");
    assert_eq!(req_2.knowledge_claims[0].subject, "act-run-check");
    assert_eq!(req_2.knowledge_claims[0].value, "build clean: 0 warnings");
    assert_eq!(
        req_2.knowledge_claims[0].status,
        KnowledgeClaimStatus::Observed
    );
    assert_eq!(req_2.knowledge_claims[0].evidence_refs, vec!["obs-1"]);

    assert_eq!(req_2.evidence_links.len(), 1);
    assert_eq!(req_2.evidence_links[0].observation_id, "obs-1");
    assert_eq!(req_2.evidence_links[0].claim_id, "claim-for-obs-1");
    assert_eq!(req_2.evidence_links[0].relation, EvidenceRelation::Supports);

    // Observation summary is also represented in provider message content
    assert!(req_2.messages.iter().any(|msg| match msg {
        ProviderMessage::User(text) => text.contains("build clean: 0 warnings"),
        _ => false,
    }));
}

// ---------------------------------------------------------------------------
// Test 13: Failure Action Observation -> Knowledge -> Next ProviderRequest
// ---------------------------------------------------------------------------

#[test]
fn test_b2_c3_closed_loop_failure_observation_propagates_to_provider_request() {
    let goal = Goal::new("Validate semantic closed loop with failure observation");
    let state = AgentState::new(goal);

    let response_1 = ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: "act-run-migrate".to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "diesel", "args": ["migration", "run"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    };
    let response_2 = ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: "act-check-logs".to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "cat", "args": ["migration.log"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    };

    let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response_1), Ok(response_2)]);
    let mut decision_source = ProviderDecisionSource::new(fake_provider);

    // Runtime returns an Action failure (e.g. non-zero exit status or error message)
    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-run-migrate",
        ActionResult::failure(
            "act-run-migrate",
            "migration error: table `users` already exists",
        ),
    );
    runtime.add_result(
        "act-check-logs",
        ActionResult::success("act-check-logs", "table already exists at version 2"),
    );

    let mut agent_loop = AgentLoop::new(state)
        .with_semantic_update_pipeline(ActionToClaimProducer, KnowledgeStoreSemanticUpdater::new());

    // Step 1: Action fails in Runtime
    let outcome_1 = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 1 must succeed even when action fails (failure is an observation)");
    assert_eq!(outcome_1, LoopStepOutcome::Continue);

    // Observation must be preserved despite failure
    let obs_1 = agent_loop
        .observation_store()
        .get("obs-1")
        .expect("failed action must still produce an authoritative observation");
    assert_eq!(obs_1.source_action_id.as_deref(), Some("act-run-migrate"));
    assert_eq!(
        obs_1.summary,
        "migration error: table `users` already exists"
    );
    assert_eq!(obs_1.kind, ObservationKind::ActionResult);

    // Semantic updater derived a claim from the failure observation
    assert_eq!(agent_loop.knowledge_store().claims().len(), 1);
    assert_eq!(
        agent_loop.knowledge_store().claims()[0].value,
        "migration error: table `users` already exists"
    );
    assert_eq!(
        agent_loop.knowledge_store().evidence()[0].observation_id,
        "obs-1"
    );

    // Step 2: Next decision context is compiled
    let outcome_2 = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 2 must succeed");
    assert_eq!(outcome_2, LoopStepOutcome::Continue);

    // Failure claim and provenance link are present in the second ProviderRequest
    let requests = decision_source.provider().recorded_requests();
    assert_eq!(requests.len(), 2);
    let req_2 = &requests[1];

    assert_eq!(req_2.knowledge_claims.len(), 1);
    assert_eq!(
        req_2.knowledge_claims[0].value,
        "migration error: table `users` already exists"
    );
    assert_eq!(req_2.evidence_links.len(), 1);
    assert_eq!(req_2.evidence_links[0].observation_id, "obs-1");
    assert_eq!(req_2.evidence_links[0].relation, EvidenceRelation::Supports);

    assert!(req_2.messages.iter().any(|msg| match msg {
        ProviderMessage::User(text) =>
            text.contains("migration error: table `users` already exists"),
        _ => false,
    }));
}

// ---------------------------------------------------------------------------
// Test 14: Semantic update failure preserves observation and maintains atomicity
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct BrokenSemanticProducer;

impl SemanticUpdateProducer for BrokenSemanticProducer {
    fn produce(&mut self, _observation: &Observation) -> Option<SemanticUpdate> {
        // Deliberately reference non-existent Observation ID to trigger KnowledgeStore invariant error
        Some(SemanticUpdate::Evidence(EvidenceLink {
            observation_id: "non-existent-observation-id".to_string(),
            claim_id: "non-existent-claim-id".to_string(),
            relation: EvidenceRelation::Supports,
        }))
    }
}

#[derive(Debug)]
struct InvalidProvenanceClaimProducer;

impl SemanticUpdateProducer for InvalidProvenanceClaimProducer {
    fn produce(&mut self, _observation: &Observation) -> Option<SemanticUpdate> {
        // Observed claim with evidence pointing to missing observation
        Some(SemanticUpdate::ClaimWithEvidence {
            claim: KnowledgeClaim {
                id: "claim-missing-obs".to_string(),
                subject: "ghost-action".to_string(),
                predicate: "ghost-pred".to_string(),
                value: "ghost-val".to_string(),
                status: KnowledgeClaimStatus::Observed,
                scope: "test".to_string(),
                evidence_refs: vec!["ghost-obs".to_string()],
            },
            evidence: vec![EvidenceLink {
                observation_id: "ghost-obs".to_string(),
                claim_id: "claim-missing-obs".to_string(),
                relation: EvidenceRelation::Supports,
            }],
        })
    }
}

#[test]
fn test_b2_c3_closed_loop_semantic_update_failure_preserves_observation_atomicity() {
    let goal = Goal::new("Validate failure atomicity and observation preservation");

    // Case A: Missing observation in evidence link
    {
        let state = AgentState::new(goal.clone());
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![ProviderToolCall {
                id: "act-run".to_string(),
                name: "execute_process".to_string(),
                arguments: serde_json::json!({ "executable": "cargo", "args": ["build"] }),
            }],
            finish_reason: Some("tool_calls".to_string()),
        };
        let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response)]);
        let mut decision_source = ProviderDecisionSource::new(fake_provider);

        let mut runtime = FakeRuntime::new();
        runtime.add_result(
            "act-run",
            ActionResult::success("act-run", "build succeeded"),
        );

        let mut agent_loop = AgentLoop::new(state).with_semantic_update_pipeline(
            BrokenSemanticProducer,
            KnowledgeStoreSemanticUpdater::new(),
        );

        let err = agent_loop
            .step(&mut decision_source, &mut runtime)
            .expect_err("step must fail when semantic update is invalid");

        assert!(matches!(err, LoopError::SemanticUpdate(_)));

        // 1. Observation was recorded prior to semantic update and remains in store
        let obs = agent_loop
            .observation_store()
            .get("obs-1")
            .expect("obs-1 must remain in authoritative observation store");
        assert_eq!(obs.source_action_id.as_deref(), Some("act-run"));
        assert_eq!(obs.summary, "build succeeded");

        // 2. KnowledgeStore is completely unmodified (atomicity preserved)
        assert!(agent_loop.knowledge_store().claims().is_empty());
        assert!(agent_loop.knowledge_store().evidence().is_empty());
        assert!(agent_loop.knowledge_store().unknowns().is_empty());
    }

    // Case B: Observed claim with missing observation provenance
    {
        let state = AgentState::new(goal);
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![ProviderToolCall {
                id: "act-run-2".to_string(),
                name: "execute_process".to_string(),
                arguments: serde_json::json!({ "executable": "cargo", "args": ["build"] }),
            }],
            finish_reason: Some("tool_calls".to_string()),
        };
        let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response)]);
        let mut decision_source = ProviderDecisionSource::new(fake_provider);

        let mut runtime = FakeRuntime::new();
        runtime.add_result("act-run-2", ActionResult::success("act-run-2", "build ok"));

        let mut agent_loop = AgentLoop::new(state).with_semantic_update_pipeline(
            InvalidProvenanceClaimProducer,
            KnowledgeStoreSemanticUpdater::new(),
        );

        let err = agent_loop
            .step(&mut decision_source, &mut runtime)
            .expect_err("step must fail when claim provenance references missing observation");

        assert!(matches!(err, LoopError::SemanticUpdate(_)));

        // Observation must still be in ObservationStore
        assert!(agent_loop.observation_store().get("obs-1").is_some());

        // KnowledgeStore must have zero claims/evidence/unknowns (atomic rollback/no insertion)
        assert!(agent_loop.knowledge_store().claims().is_empty());
        assert!(agent_loop.knowledge_store().evidence().is_empty());
        assert!(agent_loop.knowledge_store().unknowns().is_empty());
    }
}

// ---------------------------------------------------------------------------
// Test 15: Boundary & Ownership Invariants and Conflict Coexistence
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ConflictingAndUnknownProducer {
    step: usize,
}

impl SemanticUpdateProducer for ConflictingAndUnknownProducer {
    fn produce(&mut self, observation: &Observation) -> Option<SemanticUpdate> {
        self.step += 1;
        match self.step {
            1 => {
                let obs_id = observation.id.clone();
                Some(SemanticUpdate::ClaimWithEvidence {
                    claim: KnowledgeClaim {
                        id: "claim-port-8080".to_string(),
                        subject: "web-server".to_string(),
                        predicate: "listening_port".to_string(),
                        value: "8080".to_string(),
                        status: KnowledgeClaimStatus::Observed,
                        scope: "network".to_string(),
                        evidence_refs: vec![obs_id.clone()],
                    },
                    evidence: vec![EvidenceLink {
                        observation_id: obs_id,
                        claim_id: "claim-port-8080".to_string(),
                        relation: EvidenceRelation::Supports,
                    }],
                })
            }
            2 => {
                let obs_id = observation.id.clone();
                Some(SemanticUpdate::ClaimWithEvidence {
                    claim: KnowledgeClaim {
                        id: "claim-port-9090".to_string(),
                        subject: "web-server".to_string(),
                        predicate: "listening_port".to_string(),
                        value: "9090".to_string(),
                        status: KnowledgeClaimStatus::Observed,
                        scope: "network".to_string(),
                        evidence_refs: vec![obs_id.clone()],
                    },
                    evidence: vec![EvidenceLink {
                        observation_id: obs_id,
                        claim_id: "claim-port-9090".to_string(),
                        relation: EvidenceRelation::Supports,
                    }],
                })
            }
            3 => Some(SemanticUpdate::Unknown(Unknown {
                id: "unknown-tls-cert".to_string(),
                subject: "web-server".to_string(),
                scope: "security".to_string(),
                question: "is TLS 1.3 enabled?".to_string(),
            })),
            _ => None,
        }
    }
}

#[test]
fn test_b2_c3_closed_loop_boundary_and_ownership_invariants() {
    let goal = Goal::new("Validate boundary ownership and conflict coexistence");
    let state = AgentState::new(goal);

    let response_1 = ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: "act-probe-1".to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "curl", "args": ["http://localhost:8080"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    };
    let response_2 = ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: "act-probe-2".to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "curl", "args": ["http://localhost:9090"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    };
    let response_3 = ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: "act-probe-3".to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "openssl", "args": ["s_client"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    };
    let response_4 = ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: "act-probe-4".to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "echo", "args": ["done"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    };

    let fake_provider = FakeLlmProvider::with_responses(vec![
        Ok(response_1),
        Ok(response_2),
        Ok(response_3),
        Ok(response_4),
    ]);
    let mut decision_source = ProviderDecisionSource::new(fake_provider);

    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-probe-1",
        ActionResult::success("act-probe-1", "response on 8080"),
    );
    runtime.add_result(
        "act-probe-2",
        ActionResult::success("act-probe-2", "response on 9090"),
    );
    runtime.add_result(
        "act-probe-3",
        ActionResult::success("act-probe-3", "handshake completed"),
    );
    runtime.add_result("act-probe-4", ActionResult::success("act-probe-4", "done"));

    let mut agent_loop = AgentLoop::new(state).with_semantic_update_pipeline(
        ConflictingAndUnknownProducer { step: 0 },
        KnowledgeStoreSemanticUpdater::new(),
    );

    // Step 1: Probe 8080 -> Records Claim: port=8080
    agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 1 succeeds");

    // Step 2: Probe 9090 -> Records conflicting Claim: port=9090
    agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 2 succeeds");

    // Step 3: Probe TLS -> Records first-class Unknown
    agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 3 succeeds");

    // Step 4: Compiles context containing both conflicting claims and the unknown
    agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 4 succeeds");

    let requests = decision_source.provider().recorded_requests();
    assert_eq!(requests.len(), 4);
    let req_4 = &requests[3];

    // Invariant 1: Conflicting claims coexist without automatic truth resolution
    assert_eq!(req_4.knowledge_claims.len(), 2);
    let claim_values: Vec<&str> = req_4
        .knowledge_claims
        .iter()
        .map(|c| c.value.as_str())
        .collect();
    assert!(claim_values.contains(&"8080"));
    assert!(claim_values.contains(&"9090"));
    assert_eq!(req_4.evidence_links.len(), 2);

    // Invariant 2: Unknown is present as an independent first-class semantic object
    assert_eq!(req_4.knowledge_unknowns.len(), 1);
    assert_eq!(req_4.knowledge_unknowns[0].id, "unknown-tls-cert");
    assert_eq!(req_4.knowledge_unknowns[0].question, "is TLS 1.3 enabled?");

    // Invariant 3: ProviderRequest is provider-neutral and detached from authoritative store
    assert_eq!(agent_loop.knowledge_store().claims().len(), 2);
    assert_eq!(agent_loop.knowledge_store().unknowns().len(), 1);

    // Invariant 4: Structural absence of AgentDecision::UpdateKnowledge
    let decision = AgentDecision::observe("check environment");
    match decision {
        AgentDecision::Observe { .. } => {}
        AgentDecision::Act { .. } => {}
        AgentDecision::Wait { .. } => {}
        AgentDecision::Finish { .. } => {}
    }
}

// ---------------------------------------------------------------------------
// B2-C4: Failure and Atomicity Boundary Verification Tests
// ---------------------------------------------------------------------------
//
// B2-C4 is a verification-only increment. It adds no production behaviour and
// introduces no new semantic capability. These tests pin the failure and
// atomicity guarantees that B1 (KnowledgeStore invariants) and B2-C1/B2-C2
// (semantic update boundary + Observation-first ordering) already implement.

/// A scripted decision source helper: one `execute_process` action per step.
fn b2_c4_process_decision(action_id: &str) -> ProviderResponse {
    ProviderResponse {
        content: String::new(),
        tool_calls: vec![ProviderToolCall {
            id: action_id.to_string(),
            name: "execute_process".to_string(),
            arguments: serde_json::json!({ "executable": "cargo", "args": ["build"] }),
        }],
        finish_reason: Some("tool_calls".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Test 16 (T1): Multi-item atomicity — a ClaimWithEvidence carrying several
// evidence links, where a later link references a missing Observation, must
// leave the KnowledgeStore byte-identical to its pre-update snapshot.
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct MultiLinkSecondInvalidProducer;

impl SemanticUpdateProducer for MultiLinkSecondInvalidProducer {
    fn produce(&mut self, observation: &Observation) -> Option<SemanticUpdate> {
        let valid_obs = observation.id.clone();
        Some(SemanticUpdate::ClaimWithEvidence {
            claim: KnowledgeClaim {
                id: "claim-multi-link".to_string(),
                subject: observation.source_action_id.clone().unwrap_or_default(),
                predicate: "execution_result".to_string(),
                value: observation.summary.clone(),
                status: KnowledgeClaimStatus::Observed,
                scope: "b2-c4-multi-item-atomicity".to_string(),
                evidence_refs: vec![valid_obs.clone()],
            },
            evidence: vec![
                // Valid link: references the Observation that was just recorded.
                EvidenceLink {
                    observation_id: valid_obs,
                    claim_id: "claim-multi-link".to_string(),
                    relation: EvidenceRelation::Supports,
                },
                // Invalid link: references an Observation that does not exist.
                EvidenceLink {
                    observation_id: "ghost-observation".to_string(),
                    claim_id: "claim-multi-link".to_string(),
                    relation: EvidenceRelation::Supports,
                },
            ],
        })
    }
}

#[test]
fn test_b2_c4_multi_item_claim_update_is_atomic() {
    let goal = Goal::new("Verify multi-item claim update atomicity");
    let state = AgentState::new(goal);

    let response = b2_c4_process_decision("act-multi-link");
    let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response)]);
    let mut decision_source = ProviderDecisionSource::new(fake_provider);

    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-multi-link",
        ActionResult::success("act-multi-link", "build succeeded"),
    );

    let mut agent_loop = AgentLoop::new(state).with_semantic_update_pipeline(
        MultiLinkSecondInvalidProducer,
        KnowledgeStoreSemanticUpdater::new(),
    );

    let before = agent_loop.knowledge_store().clone();

    let err = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect_err("a later invalid evidence link must fail the whole update");

    assert!(
        matches!(err, LoopError::SemanticUpdate(_)),
        "expected a semantic update error, got: {err:?}"
    );

    // The failure must come specifically from the *second* (invalid) link.
    // Pinning the exact error proves the earlier, valid link was accepted into
    // the candidate state before the update was rejected as a whole.
    match &err {
        LoopError::SemanticUpdate(SemanticUpdateError::KnowledgeStore(
            KnowledgeStoreError::MissingObservation(missing),
        )) => assert_eq!(missing, "ghost-observation"),
        other => panic!("expected MissingObservation('ghost-observation'), got: {other:?}"),
    }

    // The Observation recorded before the semantic update remains historical evidence.
    let obs = agent_loop
        .observation_store()
        .get("obs-1")
        .expect("obs-1 must remain in the authoritative observation store");
    assert_eq!(obs.source_action_id.as_deref(), Some("act-multi-link"));
    assert_eq!(obs.summary, "build succeeded");

    // No partial mutation: neither the claim nor the first (valid) link is retained.
    let after = agent_loop.knowledge_store();
    assert_eq!(after, &before, "KnowledgeStore must be unchanged");
    assert!(after.claims().is_empty());
    assert!(after.evidence().is_empty());
    assert!(after.unknowns().is_empty());
}

// ---------------------------------------------------------------------------
// Test 17 (T2): Duplicate identifier at the semantic boundary.
// A repeated claim identifier fails deterministically and must not modify the
// state produced by the first successful update.
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct DuplicateClaimProducer {
    step: usize,
}

impl SemanticUpdateProducer for DuplicateClaimProducer {
    fn produce(&mut self, observation: &Observation) -> Option<SemanticUpdate> {
        self.step += 1;
        let obs_id = observation.id.clone();
        Some(SemanticUpdate::ClaimWithEvidence {
            claim: KnowledgeClaim {
                // Intentionally identical on every step.
                id: "claim-duplicate".to_string(),
                subject: observation.source_action_id.clone().unwrap_or_default(),
                predicate: "execution_result".to_string(),
                value: observation.summary.clone(),
                status: KnowledgeClaimStatus::Observed,
                scope: "b2-c4-duplicate-claim".to_string(),
                evidence_refs: vec![obs_id.clone()],
            },
            evidence: vec![EvidenceLink {
                observation_id: obs_id,
                claim_id: "claim-duplicate".to_string(),
                relation: EvidenceRelation::Supports,
            }],
        })
    }
}

#[test]
fn test_b2_c4_duplicate_claim_identifier_fails_deterministically() {
    let goal = Goal::new("Verify duplicate claim identifier rejection");
    let state = AgentState::new(goal);

    let response_1 = b2_c4_process_decision("act-dup-1");
    let response_2 = b2_c4_process_decision("act-dup-2");
    let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response_1), Ok(response_2)]);
    let mut decision_source = ProviderDecisionSource::new(fake_provider);

    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-dup-1",
        ActionResult::success("act-dup-1", "first build"),
    );
    runtime.add_result(
        "act-dup-2",
        ActionResult::success("act-dup-2", "second build"),
    );

    let mut agent_loop = AgentLoop::new(state).with_semantic_update_pipeline(
        DuplicateClaimProducer { step: 0 },
        KnowledgeStoreSemanticUpdater::new(),
    );

    // Step 1: the first update succeeds and establishes the baseline Knowledge state.
    let outcome_1 = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 1 must succeed");
    assert_eq!(outcome_1, LoopStepOutcome::Continue);

    let after_first = agent_loop.knowledge_store().clone();
    assert_eq!(after_first.claims().len(), 1);
    assert_eq!(after_first.evidence().len(), 1);

    // Step 2: the same claim identifier must be rejected deterministically.
    let err = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect_err("a duplicate claim identifier must be rejected");

    assert!(
        matches!(err, LoopError::SemanticUpdate(_)),
        "expected a semantic update error, got: {err:?}"
    );
    match &err {
        LoopError::SemanticUpdate(SemanticUpdateError::KnowledgeStore(
            KnowledgeStoreError::DuplicateClaimId(id),
        )) => assert_eq!(id, "claim-duplicate"),
        other => panic!("expected DuplicateClaimId('claim-duplicate'), got: {other:?}"),
    }

    // The Observation from the failing step is still retained as evidence.
    assert!(
        agent_loop.observation_store().get("obs-2").is_some(),
        "obs-2 must remain in the authoritative observation store"
    );

    // The first update's Knowledge state is preserved with no additional mutation.
    let after_second = agent_loop.knowledge_store();
    assert_eq!(
        after_second, &after_first,
        "KnowledgeStore must be unchanged"
    );
    assert_eq!(after_second.claims().len(), 1);
    assert_eq!(after_second.evidence().len(), 1);
    assert!(
        after_second
            .evidence()
            .iter()
            .all(|link| link.observation_id == "obs-1"),
        "no evidence link may reference the rejected second observation"
    );
}

// ---------------------------------------------------------------------------
// Test 18 (T3): Unknown update failure.
// A repeated Unknown identifier must fail without partially mutating the store,
// and the triggering Observation must still be retained.
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct DuplicateUnknownProducer;

impl SemanticUpdateProducer for DuplicateUnknownProducer {
    fn produce(&mut self, _observation: &Observation) -> Option<SemanticUpdate> {
        // Intentionally identical on every step.
        Some(SemanticUpdate::Unknown(Unknown {
            id: "unknown-duplicate".to_string(),
            subject: "web-server".to_string(),
            scope: "b2-c4-duplicate-unknown".to_string(),
            question: "Is TLS 1.3 enabled?".to_string(),
        }))
    }
}

#[test]
fn test_b2_c4_duplicate_unknown_identifier_preserves_observation() {
    let goal = Goal::new("Verify duplicate unknown identifier rejection");
    let state = AgentState::new(goal);

    let response_1 = b2_c4_process_decision("act-unknown-1");
    let response_2 = b2_c4_process_decision("act-unknown-2");
    let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response_1), Ok(response_2)]);
    let mut decision_source = ProviderDecisionSource::new(fake_provider);

    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-unknown-1",
        ActionResult::success("act-unknown-1", "first probe"),
    );
    runtime.add_result(
        "act-unknown-2",
        ActionResult::success("act-unknown-2", "second probe"),
    );

    let mut agent_loop = AgentLoop::new(state).with_semantic_update_pipeline(
        DuplicateUnknownProducer,
        KnowledgeStoreSemanticUpdater::new(),
    );

    agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect("step 1 must succeed");

    let after_first = agent_loop.knowledge_store().clone();
    assert_eq!(after_first.unknowns().len(), 1);
    assert!(after_first.claims().is_empty());
    assert!(after_first.evidence().is_empty());

    let err = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect_err("a duplicate unknown identifier must be rejected");

    assert!(
        matches!(err, LoopError::SemanticUpdate(_)),
        "expected a semantic update error, got: {err:?}"
    );
    match &err {
        LoopError::SemanticUpdate(SemanticUpdateError::KnowledgeStore(
            KnowledgeStoreError::DuplicateUnknownId(id),
        )) => assert_eq!(id, "unknown-duplicate"),
        other => panic!("expected DuplicateUnknownId('unknown-duplicate'), got: {other:?}"),
    }

    assert!(
        agent_loop.observation_store().get("obs-2").is_some(),
        "obs-2 must remain in the authoritative observation store"
    );

    let after_second = agent_loop.knowledge_store();
    assert_eq!(
        after_second, &after_first,
        "KnowledgeStore must be unchanged"
    );
    assert_eq!(after_second.unknowns().len(), 1);
    assert!(after_second.claims().is_empty());
    assert!(after_second.evidence().is_empty());
}

// ---------------------------------------------------------------------------
// Test 19 (T4): Deterministic repeated failure.
// Identical inputs must produce an identical failure outcome, and the
// KnowledgeStore must not drift between runs.
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct AlwaysInvalidEvidenceProducer;

impl SemanticUpdateProducer for AlwaysInvalidEvidenceProducer {
    fn produce(&mut self, _observation: &Observation) -> Option<SemanticUpdate> {
        Some(SemanticUpdate::Evidence(EvidenceLink {
            observation_id: "non-existent-observation-id".to_string(),
            claim_id: "non-existent-claim-id".to_string(),
            relation: EvidenceRelation::Supports,
        }))
    }
}

/// Runs one closed-loop step that is guaranteed to fail during the semantic
/// update, returning the resulting error and the resulting KnowledgeStore.
fn b2_c4_run_expected_semantic_failure(
    producer: AlwaysInvalidEvidenceProducer,
) -> (LoopError, InMemoryKnowledgeStore) {
    let state = AgentState::new(Goal::new("Verify deterministic repeated failure"));
    let response = b2_c4_process_decision("act-repeat");
    let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response)]);
    let mut decision_source = ProviderDecisionSource::new(fake_provider);

    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-repeat",
        ActionResult::success("act-repeat", "build succeeded"),
    );

    let mut agent_loop = AgentLoop::new(state)
        .with_semantic_update_pipeline(producer, KnowledgeStoreSemanticUpdater::new());

    let err = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect_err("the semantic update must fail deterministically");

    assert!(
        agent_loop.observation_store().get("obs-1").is_some(),
        "the observation must be retained even when the semantic update fails"
    );

    (err, agent_loop.knowledge_store().clone())
}

#[test]
fn test_b2_c4_repeated_identical_failure_is_deterministic() {
    let (first_error, first_knowledge) =
        b2_c4_run_expected_semantic_failure(AlwaysInvalidEvidenceProducer);
    let (second_error, second_knowledge) =
        b2_c4_run_expected_semantic_failure(AlwaysInvalidEvidenceProducer);

    // Identical inputs must produce an identical, explicitly typed error.
    assert_eq!(first_error, second_error);
    assert_eq!(first_error.to_string(), second_error.to_string());
    assert!(
        matches!(first_error, LoopError::SemanticUpdate(_)),
        "expected a semantic update error, got: {first_error:?}"
    );

    // Neither run may leave Knowledge state behind, and the two runs must agree.
    assert!(first_knowledge.claims().is_empty());
    assert!(first_knowledge.evidence().is_empty());
    assert!(first_knowledge.unknowns().is_empty());
    assert_eq!(first_knowledge, second_knowledge);
}

// ---------------------------------------------------------------------------
// Test 20 (T5): Step-level state consistency after a semantic update failure.
//
// This asserts the behaviour that already exists. It intentionally does not
// assert any recovery, retry, or continuation semantics.
// ---------------------------------------------------------------------------

#[test]
fn test_b2_c4_state_consistency_after_semantic_update_failure() {
    let goal = Goal::new("Verify state consistency after semantic failure");
    let state = AgentState::new(goal);

    let response = b2_c4_process_decision("act-state-consistency");
    let fake_provider = FakeLlmProvider::with_responses(vec![Ok(response)]);
    let mut decision_source = ProviderDecisionSource::new(fake_provider);

    let mut runtime = FakeRuntime::new();
    runtime.add_result(
        "act-state-consistency",
        ActionResult::success("act-state-consistency", "build succeeded"),
    );

    let mut agent_loop = AgentLoop::new(state).with_semantic_update_pipeline(
        AlwaysInvalidEvidenceProducer,
        KnowledgeStoreSemanticUpdater::new(),
    );

    let err = agent_loop
        .step(&mut decision_source, &mut runtime)
        .expect_err("the semantic update must fail");
    assert!(matches!(err, LoopError::SemanticUpdate(_)));

    // The authoritative Observation is retained.
    let obs = agent_loop
        .observation_store()
        .get("obs-1")
        .expect("obs-1 must remain in the authoritative observation store");
    assert_eq!(
        obs.source_action_id.as_deref(),
        Some("act-state-consistency")
    );

    // The bounded compatibility view in AgentState reflects the same step.
    let agent_state = agent_loop.state();
    assert_eq!(agent_state.recent_observations.len(), 1);
    assert_eq!(agent_state.recent_observations[0].id, "obs-1");
    assert_eq!(agent_state.recent_actions.len(), 1);
    assert_eq!(agent_state.recent_actions[0].id, "act-state-consistency");
    assert_eq!(agent_loop.current_step(), 1);

    // The AgentState remains internally consistent after the failed step.
    agent_state
        .validate()
        .expect("AgentState must remain valid after a semantic update failure");

    // No semantic state was produced.
    assert!(agent_loop.knowledge_store().is_empty());
}

// ---------------------------------------------------------------------------
// Test 21 (T6): ContextCompiler / checkpoint boundary.
// Context compilation must not mutate the KnowledgeStore, and checkpoint
// restoration must not fabricate Knowledge history or leave dangling
// evidence references. No Knowledge persistence is introduced.
// ---------------------------------------------------------------------------

#[test]
fn test_b2_c4_context_compiler_is_read_only_for_knowledge() {
    let state = AgentState::new(Goal::new("Verify context compiler read-only boundary"));

    let mut observations = InMemoryObservationStore::new();
    observations
        .record(Observation::new(
            "obs-1",
            ObservationKind::ActionResult,
            "build succeeded",
        ))
        .expect("record observation");

    let mut knowledge = InMemoryKnowledgeStore::new();
    knowledge
        .record_claim_with_evidence(
            KnowledgeClaim {
                id: "claim-readonly".to_string(),
                subject: "act-readonly".to_string(),
                predicate: "result".to_string(),
                value: "succeeded".to_string(),
                status: KnowledgeClaimStatus::Observed,
                scope: "b2-c4-read-only".to_string(),
                evidence_refs: vec!["obs-1".to_string()],
            },
            vec![EvidenceLink {
                observation_id: "obs-1".to_string(),
                claim_id: "claim-readonly".to_string(),
                relation: EvidenceRelation::Supports,
            }],
            &observations,
        )
        .expect("record claim with evidence");

    let before = knowledge.clone();

    let compiler = DefaultContextCompiler::default();
    let first = compiler
        .compile_with_knowledge(&state, &observations, &knowledge)
        .expect("compile must succeed");
    let second = compiler
        .compile_with_knowledge(&state, &observations, &knowledge)
        .expect("compile must succeed");

    // Compilation is deterministic...
    assert_eq!(first, second);
    assert_eq!(first.knowledge_claims.len(), 1);
    assert_eq!(first.evidence_links.len(), 1);

    // ...and it must not mutate the authoritative KnowledgeStore.
    assert_eq!(
        knowledge, before,
        "ContextCompiler must not mutate KnowledgeStore"
    );
    assert_eq!(knowledge.claims().len(), 1);
    assert_eq!(knowledge.evidence().len(), 1);
}

#[test]
fn test_b2_c4_checkpoint_restore_does_not_fabricate_knowledge() {
    // A checkpoint carrying Observation history but no Knowledge payload.
    let mut restored_state =
        AgentState::new(Goal::new("Verify checkpoint restore Knowledge boundary"));
    restored_state.record_observation(Observation::new(
        "obs-1",
        ObservationKind::ActionResult,
        "build succeeded",
    ));
    restored_state.record_action(Action::new("act-ckpt", ActionType::Execute));
    let checkpoint = StateCheckpoint::new("ckpt-b2-c4", 1, restored_state);

    let restored = AgentLoop::from_checkpoint(&checkpoint);

    // Observation history retained by the checkpoint is present.
    assert_eq!(restored.observation_store().len(), 1);
    assert!(restored.observation_store().get("obs-1").is_some());

    // Knowledge is NOT recovered, and nothing claims it was.
    assert!(restored.knowledge_store().is_empty());

    // Because the restored KnowledgeStore is empty, compilation cannot produce
    // a dangling evidence reference and must still succeed.
    let compiler = DefaultContextCompiler::default();
    let compiled = compiler
        .compile_with_knowledge(
            restored.state(),
            restored.observation_store(),
            restored.knowledge_store(),
        )
        .expect("compilation after checkpoint restore must succeed");

    assert!(compiled.knowledge_claims.is_empty());
    assert!(compiled.evidence_links.is_empty());
    assert!(compiled.knowledge_unknowns.is_empty());
    assert_eq!(compiled.observations.len(), 1);
}
