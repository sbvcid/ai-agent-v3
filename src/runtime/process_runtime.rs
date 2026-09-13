use crate::core::process::ProcessSpec;
use crate::core::runtime::Runtime;
use crate::core::types::{Action, ActionResult, ActionType};
use crate::runtime::process_policy::ProcessPolicy;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct ProcessRuntime {
    policy: ProcessPolicy,
}

impl Default for ProcessRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessRuntime {
    pub fn new() -> Self {
        Self {
            policy: ProcessPolicy::default(),
        }
    }

    pub fn with_policy(policy: ProcessPolicy) -> Self {
        Self { policy }
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

        if !spec.env.is_empty() {
            return ActionResult::failure(
                &action.id,
                "Custom environment variables are not supported in Stage 7B",
            );
        }

        // Apply Process Policy check before spawning
        if let Err(policy_err) = self.policy.check(&spec) {
            return ActionResult::failure(
                &action.id,
                format!("Process policy violation: {}", policy_err),
            );
        }

        let mut command = Command::new(&spec.executable);
        command.args(&spec.args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        if let Some(cwd) = spec.cwd {
            command.current_dir(cwd);
        }

        let start_time = Instant::now();

        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;
                return ActionResult {
                    action_id: action.id,
                    success: false,
                    exit_code: None,
                    stdout: None,
                    stderr: None,
                    error: Some(format!("Failed to spawn process: {}", e)),
                    output: None,
                    duration_ms: Some(duration_ms),
                    affected_resources: Vec::new(),
                };
            }
        };

        // Handle execution with optional timeout
        let result = if let Some(timeout_ms) = spec.timeout_ms {
            wait_with_timeout(&mut child, Duration::from_millis(timeout_ms))
        } else {
            wait_normal(&mut child)
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(output_data) => {
                let (status, stdout_bytes, stderr_bytes) = output_data;
                let exit_code = status.code();
                let success = status.success();

                ActionResult {
                    action_id: action.id,
                    success,
                    exit_code,
                    stdout: Some(String::from_utf8_lossy(&stdout_bytes).into_owned()),
                    stderr: Some(String::from_utf8_lossy(&stderr_bytes).into_owned()),
                    error: None,
                    output: None,
                    duration_ms: Some(duration_ms),
                    affected_resources: Vec::new(),
                }
            }
            Err(timeout_msg) => {
                // Timeout occurred: attempt termination
                let _ = child.kill();
                let _ = child.wait(); // reap child process

                let stdout_str = child.stdout.take().and_then(|mut s| {
                    let mut buf = Vec::new();
                    use std::io::Read;
                    s.read_to_end(&mut buf).ok()?;
                    Some(String::from_utf8_lossy(&buf).into_owned())
                });
                let stderr_str = child.stderr.take().and_then(|mut s| {
                    let mut buf = Vec::new();
                    use std::io::Read;
                    s.read_to_end(&mut buf).ok()?;
                    Some(String::from_utf8_lossy(&buf).into_owned())
                });

                ActionResult {
                    action_id: action.id,
                    success: false,
                    exit_code: None,
                    stdout: stdout_str,
                    stderr: stderr_str,
                    error: Some(timeout_msg),
                    output: None,
                    duration_ms: Some(duration_ms),
                    affected_resources: Vec::new(),
                }
            }
        }
    }
}

fn wait_normal(child: &mut Child) -> Result<(std::process::ExitStatus, Vec<u8>, Vec<u8>), String> {
    let mut stdout_bytes = Vec::new();
    if let Some(ref mut out) = child.stdout {
        use std::io::Read;
        let _ = out.read_to_end(&mut stdout_bytes);
    }
    let mut stderr_bytes = Vec::new();
    if let Some(ref mut err) = child.stderr {
        use std::io::Read;
        let _ = err.read_to_end(&mut stderr_bytes);
    }
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for process: {}", e))?;
    Ok((status, stdout_bytes, stderr_bytes))
}

fn wait_with_timeout(
    child: &mut Child,
    timeout: Duration,
) -> Result<(std::process::ExitStatus, Vec<u8>, Vec<u8>), String> {
    let start = Instant::now();
    let poll_interval = Duration::from_millis(10);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout_bytes = Vec::new();
                if let Some(ref mut out) = child.stdout {
                    use std::io::Read;
                    let _ = out.read_to_end(&mut stdout_bytes);
                }
                let mut stderr_bytes = Vec::new();
                if let Some(ref mut err) = child.stderr {
                    use std::io::Read;
                    let _ = err.read_to_end(&mut stderr_bytes);
                }
                return Ok((status, stdout_bytes, stderr_bytes));
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    return Err(format!(
                        "Process timed out after {} ms",
                        timeout.as_millis()
                    ));
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => {
                return Err(format!("Error trying to wait for process: {}", e));
            }
        }
    }
}
