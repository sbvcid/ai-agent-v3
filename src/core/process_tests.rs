#[cfg(test)]
mod tests {
    use crate::core::types::{Action, ActionType};
    use crate::core::ProcessSpec;
    use std::collections::BTreeMap;

    #[test]
    fn test_valid_process_spec() {
        let spec = ProcessSpec {
            executable: "cmd.exe".to_string(),
            args: vec!["/c".to_string(), "dir".to_string()],
            cwd: Some("C:\\".to_string()),
            env: BTreeMap::new(),
            timeout_ms: Some(1000),
        };
        assert!(spec.validate_contract().is_ok());
    }

    #[test]
    fn test_empty_executable() {
        let spec = ProcessSpec {
            executable: "".to_string(),
            args: vec![],
            cwd: None,
            env: BTreeMap::new(),
            timeout_ms: None,
        };
        assert!(spec.validate_contract().is_err());
    }

    #[test]
    fn test_path_traversal() {
        let spec = ProcessSpec {
            executable: "../bin/sh".to_string(),
            args: vec![],
            cwd: None,
            env: BTreeMap::new(),
            timeout_ms: None,
        };
        assert!(spec.validate_contract().is_err());
    }

    #[test]
    fn test_from_action_valid() {
        let mut action = Action::new("act-1", ActionType::Execute);
        action = action.with_parameter("executable", "git");
        action = action.with_parameter("args", r#"["status"]"#);

        let spec = ProcessSpec::from_action(&action).unwrap();
        assert_eq!(spec.executable, "git");
        assert_eq!(spec.args, vec!["status"]);
    }

    #[test]
    fn test_from_action_missing_executable() {
        let action = Action::new("act-1", ActionType::Execute);
        assert!(ProcessSpec::from_action(&action).is_err());
    }

    #[test]
    fn test_from_action_invalid_args() {
        let mut action = Action::new("act-1", ActionType::Execute);
        action = action.with_parameter("executable", "git");
        action = action.with_parameter("args", "invalid-json");
        assert!(ProcessSpec::from_action(&action).is_err());
    }
}
