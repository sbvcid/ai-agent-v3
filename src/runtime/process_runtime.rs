use std::process::Command;
use crate::core::process::ProcessSpec;
use crate::core::runtime::Runtime;
use crate::core::types::{Action, ActionResult, ActionType};

pub struct ProcessRuntime;

impl Default for ProcessRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Runtime for ProcessRuntime {
    fn execute(&mut self, action: Action) -> ActionResult {
        if action.action_type != ActionType::Execute {
            return ActionResult::failure(&action.id, "Action type must be Execute");
        }

        let spec = match ProcessSpec::from_action(&action) {
            Ok(s) => s,
            Err(e) => {
                return ActionResult::failure(
                    &action.id,
                    format!("Failed to parse process spec: {}", e),
                )
            }
        };

        if spec.timeout_ms.is_some() {
            return ActionResult::failure(
                &action.id,
                "Timeout is not supported in Stage 7B",
            );
        }

        if !spec.env.is_empty() {
            return ActionResult::failure(
                &action.id,
                "Custom environment variables are not supported in Stage 7B",
            );
        }

        let mut command = Command::new(&spec.executable);
        command.args(&spec.args);

        if let Some(cwd) = spec.cwd {
            command.current_dir(cwd);
        }

        match command.output() {
            Ok(output) => {
                let exit_code = output.status.code();
                let success = output.status.success();

                ActionResult {
                    action_id: action.id,
                    success,
                    exit_code,
                    stdout: Some(String::from_utf8_lossy(&output.stdout).into_owned()),
                    stderr: Some(String::from_utf8_lossy(&output.stderr).into_owned()),
                    error: None,
                    output: None,
                    duration_ms: None,
                    affected_resources: Vec::new(),
                }
            }
            Err(e) => ActionResult {
                action_id: action.id,
                success: false,
                exit_code: None,
                stdout: None,
                stderr: None,
                error: Some(format!("Failed to spawn process: {}", e)),
                output: None,
                duration_ms: None,
                affected_resources: Vec::new(),
            },
        }
    }
}
