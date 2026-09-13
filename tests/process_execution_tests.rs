#[cfg(test)]
mod tests {
    use ai_agent_v3::core::types::{Action, ActionType};
    use ai_agent_v3::runtime::process_runtime::ProcessRuntime;
    use ai_agent_v3::core::runtime::Runtime;

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
    fn test_process_execution_fail() {
        let mut runtime = ProcessRuntime::new();
        let mut action = Action::new("act-1", ActionType::Execute);
        action = action.with_parameter("executable", "cmd.exe");
        action = action.with_parameter("args", r#"["/c", "exit 1"]"#);

        let result = runtime.execute(action);
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(1));
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
    fn test_process_execution_invalid_action() {
        let mut runtime = ProcessRuntime::new();
        let action = Action::new("act-1", ActionType::Observe); // Wrong ActionType

        let result = runtime.execute(action);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Action type must be Execute"));
    }
}
