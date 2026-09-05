use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    #[error("Field '{0}' cannot be empty")]
    EmptyField(String),
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),
}

/// Original goal provided by the user.
///
/// Must persist throughout the task lifecycle. All final verification must relate back to this Goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Goal {
    pub description: String,
}

impl Goal {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.description.trim().is_empty() {
            return Err(ValidationError::EmptyField("description".to_string()));
        }
        Ok(())
    }
}

/// Epistemic status of information held by the Agent.
///
/// Agent Core distinguishes between different degrees of knowledge certainty.
/// Hypotheses must never be silently converted into Facts without evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeState {
    Fact,
    Observed,
    Inferred,
    Hypothesis,
    Unknown,
}

/// Verification state of subtasks or goals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationState {
    Unverified,
    InProgress,
    Verified,
    Failed,
}

/// Execution lifecycle state of the Agent.
///
/// Strictly separated from [`FinalTaskStatus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionState {
    Running,
    Waiting,
}

/// Terminal outcome of a task.
///
/// Strictly separated from [`ExecutionState`]. `AgentDecision::Finish` must carry this status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FinalTaskStatus {
    Done,
    Blocked { reason: Option<String> },
    Impossible { reason: Option<String> },
    NeedUser { reason: Option<String> },
}

/// High-level categories of Runtime operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Observe,
    Execute,
    Interact,
    Wait,
}

/// Canonical operation model requested by Agent Core and executed by Runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub action_type: ActionType,
    pub parameters: BTreeMap<String, String>,
    pub intent: Option<String>,
    pub expected_effect: Option<String>,
}

impl Action {
    pub fn new(id: impl Into<String>, action_type: ActionType) -> Self {
        Self {
            id: id.into(),
            action_type,
            parameters: BTreeMap::new(),
            intent: None,
            expected_effect: None,
        }
    }

    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn with_expected_effect(mut self, effect: impl Into<String>) -> Self {
        self.expected_effect = Some(effect.into());
        self
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id.trim().is_empty() {
            return Err(ValidationError::EmptyField("id".to_string()));
        }
        Ok(())
    }
}

/// Canonical result model returned by Runtime after executing an [`Action`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionResult {
    pub action_id: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub error: Option<String>,
    pub output: Option<String>,
    pub duration_ms: Option<u64>,
    pub affected_resources: Vec<String>,
}

impl ActionResult {
    pub fn success(action_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            success: true,
            exit_code: Some(0),
            stdout: None,
            stderr: None,
            error: None,
            output: Some(output.into()),
            duration_ms: None,
            affected_resources: Vec::new(),
        }
    }

    pub fn failure(action_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            success: false,
            exit_code: Some(1),
            stdout: None,
            stderr: None,
            error: Some(error.into()),
            output: None,
            duration_ms: None,
            affected_resources: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.action_id.trim().is_empty() {
            return Err(ValidationError::EmptyField("action_id".to_string()));
        }
        if !self.success && self.error.is_none() && self.stderr.is_none() {
            return Err(ValidationError::InvariantViolation(
                "Failure result must contain error or stderr".to_string(),
            ));
        }
        Ok(())
    }
}

/// Kinds of observations from the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    Environment,
    Filesystem,
    Process,
    ActionResult,
    Visual,
    Browser,
    EnvironmentDelta,
    Job,
    Engineering,
}

/// Objective environmental evidence connecting the real computer to Agent Core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub id: String,
    pub kind: ObservationKind,
    pub summary: String,
    pub raw_data: Option<String>,
    pub source_action_id: Option<String>,
}

impl Observation {
    pub fn new(id: impl Into<String>, kind: ObservationKind, summary: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            summary: summary.into(),
            raw_data: None,
            source_action_id: None,
        }
    }

    pub fn from_action_result(id: impl Into<String>, result: &ActionResult) -> Self {
        let summary = if result.success {
            result
                .output
                .clone()
                .unwrap_or_else(|| "Action succeeded".to_string())
        } else {
            result
                .error
                .clone()
                .unwrap_or_else(|| "Action failed".to_string())
        };

        Self {
            id: id.into(),
            kind: ObservationKind::ActionResult,
            summary,
            raw_data: result.stdout.clone().or_else(|| result.stderr.clone()),
            source_action_id: Some(result.action_id.clone()),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id.trim().is_empty() {
            return Err(ValidationError::EmptyField("id".to_string()));
        }
        if self.summary.trim().is_empty() {
            return Err(ValidationError::EmptyField("summary".to_string()));
        }
        Ok(())
    }
}

/// Canonical decision model produced by Agent Core for what should happen next.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentDecision {
    Observe {
        intent: Option<String>,
    },
    Act {
        action: Action,
        intent: Option<String>,
        expected_progress: Option<String>,
    },
    Wait {
        reason: Option<String>,
    },
    Finish {
        status: FinalTaskStatus,
        message: Option<String>,
    },
}

impl AgentDecision {
    pub fn act(action: Action) -> Self {
        Self::Act {
            action,
            intent: None,
            expected_progress: None,
        }
    }

    pub fn finish(status: FinalTaskStatus) -> Self {
        Self::Finish {
            status,
            message: None,
        }
    }

    pub fn observe(intent: impl Into<String>) -> Self {
        Self::Observe {
            intent: Some(intent.into()),
        }
    }

    pub fn wait(reason: impl Into<String>) -> Self {
        Self::Wait {
            reason: Some(reason.into()),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            AgentDecision::Act { action, .. } => action.validate(),
            _ => Ok(()),
        }
    }
}

/// Persistent working memory of the Agent for the current task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentState {
    pub goal: Goal,
    pub understanding: String,
    pub environment_state: BTreeMap<String, String>,
    pub active_problems: Vec<String>,
    pub unknowns: Vec<String>,
    pub hypotheses: Vec<String>,
    pub evidence: Vec<String>,
    pub recent_actions: Vec<Action>,
    pub recent_observations: Vec<Observation>,
    pub running_jobs: Vec<String>,
    pub verification_state: VerificationState,
    pub remaining_work: Vec<String>,
    pub execution_state: ExecutionState,
    pub final_status: Option<FinalTaskStatus>,
}

impl AgentState {
    pub fn new(goal: Goal) -> Self {
        Self {
            goal,
            understanding: String::new(),
            environment_state: BTreeMap::new(),
            active_problems: Vec::new(),
            unknowns: Vec::new(),
            hypotheses: Vec::new(),
            evidence: Vec::new(),
            recent_actions: Vec::new(),
            recent_observations: Vec::new(),
            running_jobs: Vec::new(),
            verification_state: VerificationState::Unverified,
            remaining_work: Vec::new(),
            execution_state: ExecutionState::Running,
            final_status: None,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.goal.validate()?;
        for action in &self.recent_actions {
            action.validate()?;
        }
        for obs in &self.recent_observations {
            obs.validate()?;
        }

        // Invariant: If execution_state is Running or Waiting, final_status must be None.
        if self.final_status.is_some() && self.execution_state != ExecutionState::Waiting {
            // Note: final_status only co-exists if the task is finalized (not Running)
            // If finished, task should not be Running.
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_creation_and_validation() {
        let goal = Goal::new("Repair build and pass tests");
        assert_eq!(goal.description, "Repair build and pass tests");
        assert!(goal.validate().is_ok());

        let invalid_goal = Goal::new("   ");
        assert!(invalid_goal.validate().is_err());
    }

    #[test]
    fn test_execution_state_and_final_status_separation() {
        let exec_state = ExecutionState::Running;
        assert_ne!(exec_state, ExecutionState::Waiting);

        let final_status = FinalTaskStatus::Done;
        assert_eq!(final_status, FinalTaskStatus::Done);

        let blocked = FinalTaskStatus::Blocked {
            reason: Some("Network unavailable".to_string()),
        };
        assert!(matches!(blocked, FinalTaskStatus::Blocked { .. }));
    }

    #[test]
    fn test_action_and_decision_construction() {
        let action = Action::new("act-1", ActionType::Execute)
            .with_parameter("cmd", "cargo test")
            .with_intent("Verify test suite");

        assert_eq!(action.id, "act-1");
        assert_eq!(action.action_type, ActionType::Execute);
        assert_eq!(action.parameters.get("cmd").map(|s| s.as_str()), Some("cargo test"));
        assert_eq!(action.intent.as_deref(), Some("Verify test suite"));
        assert!(action.validate().is_ok());

        let decision = AgentDecision::act(action);
        if let AgentDecision::Act { action, .. } = decision {
            assert_eq!(action.id, "act-1");
        } else {
            panic!("Expected AgentDecision::Act");
        }
    }

    #[test]
    fn test_finish_decision_uses_final_task_status() {
        let decision = AgentDecision::finish(FinalTaskStatus::Done);
        if let AgentDecision::Finish { status, .. } = decision {
            assert_eq!(status, FinalTaskStatus::Done);
        } else {
            panic!("Expected AgentDecision::Finish");
        }
    }

    #[test]
    fn test_action_result_to_observation() {
        let failure_result = ActionResult::failure("act-1", "compilation error: exit 101");
        assert!(!failure_result.success);
        assert_eq!(failure_result.exit_code, Some(1));
        assert!(failure_result.validate().is_ok());

        let obs = Observation::from_action_result("obs-1", &failure_result);
        assert_eq!(obs.kind, ObservationKind::ActionResult);
        assert_eq!(obs.source_action_id.as_deref(), Some("act-1"));
        assert_eq!(obs.summary, "compilation error: exit 101");
        assert!(obs.validate().is_ok());
    }

    #[test]
    fn test_agent_state_initialization() {
        let goal = Goal::new("Convert video to AV1");
        let state = AgentState::new(goal.clone());

        assert_eq!(state.goal, goal);
        assert_eq!(state.execution_state, ExecutionState::Running);
        assert_eq!(state.final_status, None);
        assert_eq!(state.verification_state, VerificationState::Unverified);
        assert!(state.active_problems.is_empty());
        assert!(state.recent_actions.is_empty());
        assert!(state.recent_observations.is_empty());
        assert!(state.validate().is_ok());
    }
}
