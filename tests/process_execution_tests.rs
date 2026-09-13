use ai_agent_v3::core::runtime::Runtime;
use ai_agent_v3::core::types::{Action, ActionType};
use ai_agent_v3::runtime::process_policy::ProcessPolicy;
use ai_agent_v3::runtime::process_runtime::ProcessRuntime;
use std::path::PathBuf;

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
    assert!(result.error.is_none());
}

#[test]
fn test_process_execution_spawn_fail() {
    let policy = ProcessPolicy::new(
        ["non_existent_exe_12345".to_string()].into_iter().collect(),
        None,
    );
    let mut custom_runtime = ProcessRuntime::with_policy(policy);
    let mut spawn_action = Action::new("act-spawn", ActionType::Execute);
    spawn_action = spawn_action.with_parameter("executable", "non_existent_exe_12345");

    let res = custom_runtime.execute(spawn_action);
    assert!(!res.success);
    assert!(res.error.is_some());
    assert!(res.error.unwrap().contains("Failed to spawn process"));
}

#[test]
fn test_denied_executable_never_runs() {
    let mut runtime = ProcessRuntime::new();
    let mut action = Action::new("act-policy", ActionType::Execute);
    action = action.with_parameter("executable", "powershell.exe");
    action = action.with_parameter("args", r#"["Get-Process"]"#);

    let result = runtime.execute(action);
    assert!(!result.success);
    assert!(result.error.is_some());
    let err_msg = result.error.unwrap();
    assert!(err_msg.contains("Process policy violation"));
    assert!(err_msg.contains("powershell.exe"));
}

#[test]
fn test_denied_cwd_never_spawns() {
    let root = PathBuf::from("C:\\agent\\workspace");
    let policy = ProcessPolicy::new(["cmd.exe".to_string()].into_iter().collect(), Some(root));
    let mut runtime = ProcessRuntime::with_policy(policy);
    let mut action = Action::new("act-cwd", ActionType::Execute);
    action = action.with_parameter("executable", "cmd.exe");
    action = action.with_parameter("cwd", "C:\\agent\\workspace-evil");

    let result = runtime.execute(action);
    assert!(!result.success);
    assert!(result.error.is_some());
    let err_msg = result.error.unwrap();
    assert!(err_msg.contains("Process policy violation"));
    assert!(err_msg.contains("workspace-evil"));
}

#[test]
fn test_policy_failure_distinguishable_from_spawn_failure() {
    let mut runtime = ProcessRuntime::new();
    let mut action = Action::new("act-distinguish", ActionType::Execute);
    action = action.with_parameter("executable", "forbidden.exe");

    let result = runtime.execute(action);
    assert!(!result.success);
    let err = result.error.unwrap();
    assert!(err.contains("Process policy violation"));
    assert!(!err.contains("Failed to spawn process"));
}

#[test]
fn test_process_execution_invalid_action() {
    let mut runtime = ProcessRuntime::new();
    let action = Action::new("act-1", ActionType::Observe);

    let result = runtime.execute(action);
    assert!(!result.success);
    assert!(result
        .error
        .unwrap()
        .contains("Action type must be Execute"));
}

#[test]
fn test_process_execution_timeout_completes_before_deadline() {
    let mut runtime = ProcessRuntime::new();
    let mut action = Action::new("act-fast", ActionType::Execute);
    action = action.with_parameter("executable", "cmd.exe");
    action = action.with_parameter("args", r#"["/c", "echo quick"]"#);
    action = action.with_parameter("timeout_ms", "5000");

    let result = runtime.execute(action);
    assert!(result.success);
    assert_eq!(result.exit_code, Some(0));
    assert!(result.stdout.unwrap().contains("quick"));
    assert!(result.duration_ms.is_some());
}

#[test]
fn test_process_execution_timeout_exceeded() {
    let mut runtime = ProcessRuntime::new();
    let mut action = Action::new("act-slow", ActionType::Execute);
    action = action.with_parameter("executable", "cmd.exe");
    // Ping localhost to sleep for ~3-4 seconds, but set timeout to 200ms
    action = action.with_parameter("args", r#"["/c", "ping 127.0.0.1 -n 4 > nul"]"#);
    action = action.with_parameter("timeout_ms", "200");

    let result = runtime.execute(action);
    assert!(!result.success);
    assert_eq!(result.exit_code, None);
    assert!(result.error.is_some());
    let err = result.error.unwrap();
    assert!(err.contains("timed out"));
    assert!(result.duration_ms.is_some());
}
