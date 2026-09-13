#[cfg(test)]
mod tests {
    use ai_agent_v3::core::runtime::Runtime;
    use ai_agent_v3::core::types::{Action, ActionType};
    use ai_agent_v3::runtime::process_runtime::ProcessRuntime;

    #[test]
    fn test_process_execution_echo_success() {
        let mut runtime = ProcessRuntime::new();
        let mut action = Action::new("act-1", ActionType::Execute);
        action = action.with_parameter("executable", "cmd.exe");
        action = action.with_parameter("args", r#"["/c", "echo hello"]"#);

        let result = runtime.execute(action);
        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout.unwrap().contains("hello"));
    }

    #[test]
    fn test_process_execution_args_passing() {
        let mut runtime = ProcessRuntime::new();
        let mut action = Action::new("act-args", ActionType::Execute);
        action = action.with_parameter("executable", "cmd.exe");
        action = action.with_parameter("args", r#"["/c", "echo arg1_value arg2_value"]"#);

        let result = runtime.execute(action);
        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
        let stdout = result.stdout.unwrap();
        assert!(stdout.contains("arg1_value"));
        assert!(stdout.contains("arg2_value"));
    }

    #[test]
    fn test_process_execution_fail() {
        let mut runtime = ProcessRuntime::new();
        let mut action = Action::new("act-1", ActionType::Execute);
        action = action.with_parameter("executable", "cmd.exe");
        action = action.with_parameter("args", r#"["/c", "exit 1"]"#);

        let result = runtime.execute(action);
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(1));
        assert!(result.error.is_none()); // Non-zero exit code does not populate `error`
    }

    #[test]
    fn test_process_execution_spawn_fail() {
        let mut runtime = ProcessRuntime::new();
        let mut action = Action::new("act-1", ActionType::Execute);
        action = action.with_parameter("executable", "non_existent_exe_12345");

        let result = runtime.execute(action);
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Failed to spawn process"));
    }

    #[test]
    fn test_process_execution_timeout_rejected() {
        let mut runtime = ProcessRuntime::new();
        let mut action = Action::new("act-timeout", ActionType::Execute);
        action = action.with_parameter("executable", "cmd.exe");
        action = action.with_parameter("timeout_ms", "1000");

        let result = runtime.execute(action);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Timeout is not supported"));
    }

    #[test]
    fn test_process_execution_env_rejected() {
        let mut runtime = ProcessRuntime::new();
        let mut action = Action::new("act-env", ActionType::Execute);
        action = action.with_parameter("executable", "cmd.exe");
        action = action.with_parameter("env", r#"{"FOO": "bar"}"#);

        let result = runtime.execute(action);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Custom environment variables are not supported"));
    }

    #[test]
    fn test_process_execution_invalid_action() {
        let mut runtime = ProcessRuntime::new();
        let action = Action::new("act-1", ActionType::Observe);

        let result = runtime.execute(action);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Action type must be Execute"));
    }
}
