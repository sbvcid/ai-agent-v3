use ai_agent_v3::{
    Action, ActionResult, ActionType, AgentDecision, AgentState, CheckpointStore, ExecutionState,
    FinalTaskStatus, Goal, JsonFileCheckpointStore, Observation, ObservationKind, StateCheckpoint,
    VerificationState,
};
use std::fs;

#[test]
fn test_core_state_round_trip() {
    let goal = Goal::new("Repair build and pass tests");
    let mut state = AgentState::new(goal.clone());
    state.understanding = "Analyzing Cargo.toml dependency mismatch".to_string();
    state
        .environment_state
        .insert("rustc_version".to_string(), "1.80.0".to_string());
    state
        .active_problems
        .push("Cannot compile crate X".to_string());

    let action = Action::new("act-1", ActionType::Execute)
        .with_parameter("cmd", "cargo check")
        .with_intent("Check compilation errors");
    state.recent_actions.push(action);

    let obs = Observation::new("obs-1", ObservationKind::Process, "exit code 101");
    state.recent_observations.push(obs);

    // Round-trip to JSON and back
    let json_str = serde_json::to_string_pretty(&state).expect("Serialization failed");
    let deserialized_state: AgentState =
        serde_json::from_str(&json_str).expect("Deserialization failed");

    assert_eq!(state, deserialized_state);
    assert!(deserialized_state.validate().is_ok());
}

#[test]
fn test_enum_variant_round_trip() {
    // 1. ExecutionState
    for state in [ExecutionState::Running, ExecutionState::Waiting] {
        let json = serde_json::to_string(&state).unwrap();
        let de: ExecutionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, de);
    }

    // 2. FinalTaskStatus
    let statuses = vec![
        FinalTaskStatus::Done,
        FinalTaskStatus::Blocked {
            reason: Some("Waiting for external service".to_string()),
        },
        FinalTaskStatus::Impossible {
            reason: Some("Kernel access not supported".to_string()),
        },
        FinalTaskStatus::NeedUser {
            reason: Some("Requires elevated password".to_string()),
        },
    ];
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let de: FinalTaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, de);
    }

    // 3. ActionType
    for act_type in [
        ActionType::Observe,
        ActionType::Execute,
        ActionType::Interact,
        ActionType::Wait,
    ] {
        let json = serde_json::to_string(&act_type).unwrap();
        let de: ActionType = serde_json::from_str(&json).unwrap();
        assert_eq!(act_type, de);
    }

    // 4. AgentDecision
    let action = Action::new("act-2", ActionType::Observe);
    let decisions = vec![
        AgentDecision::observe("Inspect directory structure"),
        AgentDecision::act(action),
        AgentDecision::wait("Waiting on long-running build job"),
        AgentDecision::finish(FinalTaskStatus::Done),
    ];
    for dec in decisions {
        let json = serde_json::to_string(&dec).unwrap();
        let de: AgentDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(dec, de);
    }
}

#[test]
fn test_missing_required_fields_fails_deserialization() {
    // Goal missing 'description'
    let missing_goal_json = r#"{}"#;
    assert!(serde_json::from_str::<Goal>(missing_goal_json).is_err());

    // Action missing 'action_type'
    let invalid_action_json = r#"{
        "id": "act-1",
        "parameters": {}
    }"#;
    assert!(serde_json::from_str::<Action>(invalid_action_json).is_err());

    // StateCheckpoint missing 'state'
    let missing_checkpoint_state = r#"{
        "checkpoint_id": "ckpt-1",
        "timestamp_epoch_ms": 1000,
        "step_index": 1
    }"#;
    assert!(serde_json::from_str::<StateCheckpoint>(missing_checkpoint_state).is_err());
}

#[test]
fn test_strict_validation_and_unknown_fields_rejection() {
    // Strict schema: deny_unknown_fields must reject unknown attributes
    let hallucinated_json = r#"{
        "description": "Fix bug",
        "hallucinated_property": "should_fail"
    }"#;
    let result = serde_json::from_str::<Goal>(hallucinated_json);
    assert!(result.is_err(), "Expected rejection of unknown fields");

    // Invariant validation: Empty description rejected
    let empty_goal = Goal::new("   ");
    assert!(empty_goal.validate().is_err());

    // Invariant validation: ActionResult failure without error details rejected
    let invalid_result = ActionResult {
        action_id: "act-1".to_string(),
        success: false,
        exit_code: Some(1),
        stdout: None,
        stderr: None,
        error: None,
        output: None,
        duration_ms: None,
        affected_resources: vec![],
    };
    assert!(invalid_result.validate().is_err());
}

#[test]
fn test_checkpoint_save_load_round_trip() {
    let temp_dir = std::env::temp_dir().join(format!("test_ckpt_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = JsonFileCheckpointStore::new(&temp_dir).expect("Failed to create store");

    let goal = Goal::new("Build test application");
    let state = AgentState::new(goal);
    let checkpoint = StateCheckpoint::new("ckpt-roundtrip-1", 42, state).with_timestamp(1700000000);

    // Save
    store.save_checkpoint(&checkpoint).expect("Save failed");

    // Load by id
    let loaded = store
        .load_checkpoint("ckpt-roundtrip-1")
        .expect("Load by id failed");
    assert_eq!(checkpoint, loaded);
    assert_eq!(loaded.step_index, 42);

    // Load latest
    let latest = store
        .load_latest_checkpoint()
        .expect("Load latest failed")
        .expect("No latest checkpoint found");
    assert_eq!(checkpoint, latest);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_simulated_crash_and_context_restoration() {
    let temp_dir = std::env::temp_dir().join(format!("test_crash_recovery_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let goal_desc = "Process large dataset";

    // --- Process Context 1 (Before Crash) ---
    {
        let store = JsonFileCheckpointStore::new(&temp_dir).expect("Init store 1");
        let goal = Goal::new(goal_desc);
        let mut state = AgentState::new(goal);

        // Step 1: Execute action
        let act1 = Action::new("act-001", ActionType::Execute)
            .with_parameter("task", "init_data")
            .with_intent("Initialize data files");
        state.recent_actions.push(act1);
        state.recent_observations.push(Observation::new(
            "obs-001",
            ObservationKind::ActionResult,
            "Data files ready",
        ));

        // Save checkpoint before simulated crash
        let ckpt = StateCheckpoint::new("ckpt-step-1", 1, state).with_timestamp(12345678);
        store
            .save_checkpoint(&ckpt)
            .expect("Save step 1 checkpoint");

        // Context 1 drops here (simulating crash / memory loss)
    }

    // --- Process Context 2 (After Crash Recovery) ---
    {
        // Re-initialize from persistent disk storage
        let store = JsonFileCheckpointStore::new(&temp_dir).expect("Init store 2");

        // Load latest checkpoint
        let recovered_ckpt = store
            .load_latest_checkpoint()
            .expect("Load latest")
            .expect("Must have recovered checkpoint");

        assert_eq!(recovered_ckpt.checkpoint_id, "ckpt-step-1");
        assert_eq!(recovered_ckpt.step_index, 1);

        let mut restored_state = recovered_ckpt.state;
        assert_eq!(restored_state.goal.description, goal_desc);
        assert_eq!(restored_state.recent_actions.len(), 1);
        assert_eq!(restored_state.recent_actions[0].id, "act-001");
        assert_eq!(restored_state.recent_observations.len(), 1);
        assert_eq!(restored_state.recent_observations[0].id, "obs-001");
        assert_eq!(restored_state.execution_state, ExecutionState::Running);

        // Continue execution: Step 2
        let act2 = Action::new("act-002", ActionType::Execute)
            .with_parameter("task", "finalize")
            .with_intent("Finalize processing");
        restored_state.recent_actions.push(act2);
        restored_state.verification_state = VerificationState::Verified;
        restored_state.finish(FinalTaskStatus::Done);

        // Save new checkpoint
        let ckpt2 = StateCheckpoint::new("ckpt-step-2", 2, restored_state).with_timestamp(12345679);
        store
            .save_checkpoint(&ckpt2)
            .expect("Save step 2 checkpoint");

        // Verify latest updated
        let latest_after_resume = store
            .load_latest_checkpoint()
            .expect("Load latest")
            .expect("Must exist");
        assert_eq!(latest_after_resume.checkpoint_id, "ckpt-step-2");
        assert_eq!(latest_after_resume.step_index, 2);
        assert_eq!(
            latest_after_resume.state.final_status,
            Some(FinalTaskStatus::Done)
        );
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}
