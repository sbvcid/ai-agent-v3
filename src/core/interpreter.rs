//! Stage 9.1 — Core-side ProviderResponse interpreter.
//!
//! Translates a [`ProviderResponse`] from the provider layer into an
//! [`AgentDecision`] that Agent Core can act on.
//!
//! # Supported tools
//! - `execute_process` → `AgentDecision::Act` with `ActionType::Execute`
//! - `read_file`        → `AgentDecision::Act` with `ActionType::Observe`
//!
//! # Invariants
//! - Only exactly one tool call is accepted per response.
//! - Arguments must be a JSON object.
//! - Content-only responses (no tool calls) are always rejected.
//! - `finish_reason` is never used to infer `Done` or any terminal status.
//! - The interpreter does not execute Runtime, filesystem, or process operations.

use crate::core::types::{Action, ActionType, AgentDecision};
use crate::provider::{ProviderResponse, ProviderToolCall};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced when a [`ProviderResponse`] cannot be interpreted as an
/// [`AgentDecision`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InterpretationError {
    /// No tool calls and content is empty or only whitespace.
    #[error("Provider response contains no tool call and no content")]
    EmptyResponse,

    /// No tool calls but content is non-empty.
    #[error(
        "Provider response contains content but no tool call; \
         refusing to infer intent from free-form text"
    )]
    ContentWithoutToolCall,

    /// More than one tool call was present.
    #[error("Multiple tool calls are not supported; got {0} tool calls")]
    MultipleToolCalls(usize),

    /// The named tool is not recognised by this interpreter.
    #[error("Unsupported tool: '{0}'")]
    UnsupportedTool(String),

    /// The `arguments` field of the tool call was not a JSON object.
    #[error("Tool call arguments must be a JSON object")]
    ArgumentsNotObject,

    /// A required argument was absent or empty.
    #[error("Missing required argument '{0}'")]
    MissingArgument(String),

    /// An argument was present but had the wrong type or format.
    #[error("Invalid argument format for '{field}': {reason}")]
    InvalidArgumentFormat { field: String, reason: String },
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Interpret a [`ProviderResponse`] into an [`AgentDecision`].
///
/// Returns `Err(InterpretationError)` for any response that cannot be
/// unambiguously converted into one of the supported tool actions.
pub fn interpret(response: &ProviderResponse) -> Result<AgentDecision, InterpretationError> {
    match response.tool_calls.len() {
        0 => {
            if response.content.trim().is_empty() {
                Err(InterpretationError::EmptyResponse)
            } else {
                Err(InterpretationError::ContentWithoutToolCall)
            }
        }
        1 => interpret_single_tool_call(&response.tool_calls[0]),
        n => Err(InterpretationError::MultipleToolCalls(n)),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn interpret_single_tool_call(tc: &ProviderToolCall) -> Result<AgentDecision, InterpretationError> {
    let args = match &tc.arguments {
        serde_json::Value::Object(map) => map,
        _ => return Err(InterpretationError::ArgumentsNotObject),
    };

    match tc.name.as_str() {
        "execute_process" => interpret_execute_process(&tc.id, args),
        "read_file" => interpret_read_file(&tc.id, args),
        other => Err(InterpretationError::UnsupportedTool(other.to_string())),
    }
}

/// Build an `Action` for `execute_process`.
///
/// Parameter encoding compatible with [`ProcessSpec::from_action`]:
/// - `executable` → plain string (required)
/// - `args`       → JSON-serialised `Vec<String>` (optional)
/// - `cwd`        → plain string (optional)
/// - `env`        → JSON-serialised `BTreeMap<String, String>` (optional)
/// - `timeout_ms` → decimal string of a `u64` (optional)
fn interpret_execute_process(
    tool_call_id: &str,
    args: &serde_json::Map<String, serde_json::Value>,
) -> Result<AgentDecision, InterpretationError> {
    let mut parameters: BTreeMap<String, String> = BTreeMap::new();

    // --- executable (required) ---
    let executable = require_string_arg(args, "executable")?;
    if executable.trim().is_empty() {
        return Err(InterpretationError::MissingArgument(
            "executable".to_string(),
        ));
    }
    parameters.insert("executable".to_string(), executable);

    // --- args (optional, must be array of strings) ---
    if let Some(args_val) = args.get("args") {
        match args_val {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if !item.is_string() {
                        return Err(InterpretationError::InvalidArgumentFormat {
                            field: "args".to_string(),
                            reason: "array elements must be strings".to_string(),
                        });
                    }
                }
                let json_str = serde_json::to_string(args_val).map_err(|e| {
                    InterpretationError::InvalidArgumentFormat {
                        field: "args".to_string(),
                        reason: e.to_string(),
                    }
                })?;
                parameters.insert("args".to_string(), json_str);
            }
            _ => {
                return Err(InterpretationError::InvalidArgumentFormat {
                    field: "args".to_string(),
                    reason: "must be an array of strings".to_string(),
                });
            }
        }
    }

    // --- cwd (optional, must be string) ---
    if let Some(cwd_val) = args.get("cwd") {
        match cwd_val.as_str() {
            Some(cwd) => {
                parameters.insert("cwd".to_string(), cwd.to_string());
            }
            None => {
                return Err(InterpretationError::InvalidArgumentFormat {
                    field: "cwd".to_string(),
                    reason: "must be a string".to_string(),
                });
            }
        }
    }

    // --- env (optional, must be object with string values) ---
    if let Some(env_val) = args.get("env") {
        match env_val {
            serde_json::Value::Object(env_map) => {
                for (k, v) in env_map {
                    if !v.is_string() {
                        return Err(InterpretationError::InvalidArgumentFormat {
                            field: "env".to_string(),
                            reason: format!("value for key '{}' must be a string", k),
                        });
                    }
                }
                let json_str = serde_json::to_string(env_val).map_err(|e| {
                    InterpretationError::InvalidArgumentFormat {
                        field: "env".to_string(),
                        reason: e.to_string(),
                    }
                })?;
                parameters.insert("env".to_string(), json_str);
            }
            _ => {
                return Err(InterpretationError::InvalidArgumentFormat {
                    field: "env".to_string(),
                    reason: "must be an object with string values".to_string(),
                });
            }
        }
    }

    // --- timeout_ms (optional, must be non-negative integer) ---
    if let Some(timeout_val) = args.get("timeout_ms") {
        match timeout_val.as_u64() {
            Some(ms) => {
                parameters.insert("timeout_ms".to_string(), ms.to_string());
            }
            None => {
                return Err(InterpretationError::InvalidArgumentFormat {
                    field: "timeout_ms".to_string(),
                    reason: "must be a non-negative integer".to_string(),
                });
            }
        }
    }

    let action = Action {
        id: tool_call_id.to_string(),
        action_type: ActionType::Execute,
        parameters,
        intent: None,
        expected_effect: None,
    };

    Ok(AgentDecision::act(action))
}

/// Build an `Action` for `read_file`.
///
/// Parameter encoding:
/// - `op`   → fixed value `"read_file"` (consumed by FilesystemRuntime)
/// - `path` → plain string (required)
fn interpret_read_file(
    tool_call_id: &str,
    args: &serde_json::Map<String, serde_json::Value>,
) -> Result<AgentDecision, InterpretationError> {
    let path = require_string_arg(args, "path")?;

    let mut parameters: BTreeMap<String, String> = BTreeMap::new();
    parameters.insert("op".to_string(), "read_file".to_string());
    parameters.insert("path".to_string(), path);

    let action = Action {
        id: tool_call_id.to_string(),
        action_type: ActionType::Observe,
        parameters,
        intent: None,
        expected_effect: None,
    };

    Ok(AgentDecision::act(action))
}

/// Extract a required string argument from a JSON object map.
fn require_string_arg(
    args: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<String, InterpretationError> {
    match args.get(field) {
        None => Err(InterpretationError::MissingArgument(field.to_string())),
        Some(val) => match val.as_str() {
            Some(s) => Ok(s.to_string()),
            None => Err(InterpretationError::InvalidArgumentFormat {
                field: field.to_string(),
                reason: "must be a string".to_string(),
            }),
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::process::ProcessSpec;
    use crate::core::types::{Action, ActionType, AgentDecision};
    use crate::provider::{ProviderResponse, ProviderToolCall};
    use serde_json::json;

    fn make_response(name: &str, arguments: serde_json::Value) -> ProviderResponse {
        ProviderResponse {
            content: String::new(),
            tool_calls: vec![ProviderToolCall {
                id: "tc-1".to_string(),
                name: name.to_string(),
                arguments,
            }],
            finish_reason: None,
        }
    }

    fn extract_action(decision: AgentDecision) -> Action {
        match decision {
            AgentDecision::Act { action, .. } => action,
            other => panic!("Expected Act decision, got: {:?}", other),
        }
    }

    #[test]
    fn test_valid_execute_process_minimal() {
        let response = make_response("execute_process", json!({ "executable": "cmd.exe" }));
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        assert_eq!(action.action_type, ActionType::Execute);
        assert_eq!(action.parameters.get("executable").unwrap(), "cmd.exe");
        assert_eq!(action.id, "tc-1");
    }

    #[test]
    fn test_valid_execute_process_full() {
        let response = make_response(
            "execute_process",
            json!({
                "executable": "git",
                "args": ["status", "--short"],
                "cwd": "C:\\workspace",
                "env": { "GIT_TERMINAL_PROMPT": "0" },
                "timeout_ms": 5000
            }),
        );
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        assert_eq!(action.action_type, ActionType::Execute);
        assert_eq!(action.parameters.get("executable").unwrap(), "git");
        assert!(action.parameters.contains_key("args"));
        assert!(action.parameters.contains_key("env"));
        assert_eq!(action.parameters.get("timeout_ms").unwrap(), "5000");
    }

    #[test]
    fn test_valid_read_file() {
        let response = make_response("read_file", json!({ "path": "src/main.rs" }));
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        assert_eq!(action.action_type, ActionType::Observe);
        assert_eq!(action.parameters.get("op").unwrap(), "read_file");
        assert_eq!(action.parameters.get("path").unwrap(), "src/main.rs");
        assert_eq!(action.id, "tc-1");
    }

    #[test]
    fn test_args_encoding_compatible_with_process_spec() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "git", "args": ["log", "--oneline", "-5"] }),
        );
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        let spec =
            ProcessSpec::from_action(&action).expect("ProcessSpec::from_action must succeed");
        assert_eq!(spec.executable, "git");
        assert_eq!(spec.args, vec!["log", "--oneline", "-5"]);
    }

    #[test]
    fn test_empty_args_encoding() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "cmd.exe", "args": [] }),
        );
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        assert_eq!(action.parameters.get("args").unwrap(), "[]");
        let spec =
            ProcessSpec::from_action(&action).expect("ProcessSpec::from_action must succeed");
        assert!(spec.args.is_empty());
    }

    #[test]
    fn test_env_encoding_compatible_with_process_spec() {
        let response = make_response(
            "execute_process",
            json!({
                "executable": "git",
                "env": { "HOME": "C:\\Users\\agent", "GIT_AUTHOR_NAME": "Agent" }
            }),
        );
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        let spec =
            ProcessSpec::from_action(&action).expect("ProcessSpec::from_action must succeed");
        assert_eq!(spec.env.get("HOME").unwrap(), "C:\\Users\\agent");
        assert_eq!(spec.env.get("GIT_AUTHOR_NAME").unwrap(), "Agent");
    }

    #[test]
    fn test_timeout_ms_encoding_compatible_with_process_spec() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "cmd.exe", "timeout_ms": 3000 }),
        );
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        assert_eq!(action.parameters.get("timeout_ms").unwrap(), "3000");
        let spec =
            ProcessSpec::from_action(&action).expect("ProcessSpec::from_action must succeed");
        assert_eq!(spec.timeout_ms, Some(3000));
    }

    #[test]
    fn test_unsupported_tool() {
        let response = make_response("write_file", json!({ "path": "foo.txt" }));
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(err, InterpretationError::UnsupportedTool(ref n) if n == "write_file"));
    }

    #[test]
    fn test_arguments_not_object_array() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![ProviderToolCall {
                id: "tc-1".to_string(),
                name: "execute_process".to_string(),
                arguments: json!(["executable", "cmd.exe"]),
            }],
            finish_reason: None,
        };
        assert!(matches!(
            interpret(&response).expect_err("fail"),
            InterpretationError::ArgumentsNotObject
        ));
    }

    #[test]
    fn test_arguments_not_object_string() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![ProviderToolCall {
                id: "tc-1".to_string(),
                name: "execute_process".to_string(),
                arguments: json!("cmd.exe"),
            }],
            finish_reason: None,
        };
        assert!(matches!(
            interpret(&response).expect_err("fail"),
            InterpretationError::ArgumentsNotObject
        ));
    }

    #[test]
    fn test_arguments_null_is_not_object() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![ProviderToolCall {
                id: "tc-1".to_string(),
                name: "execute_process".to_string(),
                arguments: serde_json::Value::Null,
            }],
            finish_reason: None,
        };
        assert!(matches!(
            interpret(&response).expect_err("fail"),
            InterpretationError::ArgumentsNotObject
        ));
    }

    #[test]
    fn test_missing_executable() {
        let response = make_response("execute_process", json!({ "args": ["status"] }));
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(err, InterpretationError::MissingArgument(ref f) if f == "executable"));
    }

    #[test]
    fn test_empty_executable_is_missing() {
        let response = make_response("execute_process", json!({ "executable": "   " }));
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(err, InterpretationError::MissingArgument(ref f) if f == "executable"));
    }

    #[test]
    fn test_missing_path_for_read_file() {
        let response = make_response("read_file", json!({}));
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(err, InterpretationError::MissingArgument(ref f) if f == "path"));
    }

    #[test]
    fn test_invalid_args_not_array() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "git", "args": "status" }),
        );
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(
            err,
            InterpretationError::InvalidArgumentFormat { ref field, .. } if field == "args"
        ));
    }

    #[test]
    fn test_invalid_args_non_string_elements() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "git", "args": ["status", 42] }),
        );
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(
            err,
            InterpretationError::InvalidArgumentFormat { ref field, .. } if field == "args"
        ));
    }

    #[test]
    fn test_invalid_env_not_object() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "git", "env": ["KEY=VALUE"] }),
        );
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(
            err,
            InterpretationError::InvalidArgumentFormat { ref field, .. } if field == "env"
        ));
    }

    #[test]
    fn test_invalid_env_non_string_value() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "git", "env": { "KEY": 42 } }),
        );
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(
            err,
            InterpretationError::InvalidArgumentFormat { ref field, .. } if field == "env"
        ));
    }

    #[test]
    fn test_invalid_timeout_ms_negative() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "cmd.exe", "timeout_ms": -1 }),
        );
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(
            err,
            InterpretationError::InvalidArgumentFormat { ref field, .. } if field == "timeout_ms"
        ));
    }

    #[test]
    fn test_invalid_timeout_ms_string_value() {
        let response = make_response(
            "execute_process",
            json!({ "executable": "cmd.exe", "timeout_ms": "5000" }),
        );
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(
            err,
            InterpretationError::InvalidArgumentFormat { ref field, .. } if field == "timeout_ms"
        ));
    }

    #[test]
    fn test_multiple_tool_calls() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![
                ProviderToolCall {
                    id: "tc-1".to_string(),
                    name: "execute_process".to_string(),
                    arguments: json!({ "executable": "git" }),
                },
                ProviderToolCall {
                    id: "tc-2".to_string(),
                    name: "read_file".to_string(),
                    arguments: json!({ "path": "README.md" }),
                },
            ],
            finish_reason: None,
        };
        let err = interpret(&response).expect_err("should fail");
        assert!(matches!(err, InterpretationError::MultipleToolCalls(2)));
    }

    #[test]
    fn test_empty_response_no_content() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![],
            finish_reason: None,
        };
        assert!(matches!(
            interpret(&response).expect_err("fail"),
            InterpretationError::EmptyResponse
        ));
    }

    #[test]
    fn test_empty_response_only_whitespace() {
        let response = ProviderResponse {
            content: "   \n\t  ".to_string(),
            tool_calls: vec![],
            finish_reason: None,
        };
        assert!(matches!(
            interpret(&response).expect_err("fail"),
            InterpretationError::EmptyResponse
        ));
    }

    #[test]
    fn test_content_only_response_is_rejected() {
        let response = ProviderResponse {
            content: "I think you should run git status".to_string(),
            tool_calls: vec![],
            finish_reason: None,
        };
        assert!(matches!(
            interpret(&response).expect_err("fail"),
            InterpretationError::ContentWithoutToolCall
        ));
    }

    #[test]
    fn test_content_only_with_finish_reason_stop_is_rejected() {
        // finish_reason must NOT be used to infer Done or any terminal status
        let response = ProviderResponse {
            content: "Task complete".to_string(),
            tool_calls: vec![],
            finish_reason: Some("stop".to_string()),
        };
        assert!(matches!(
            interpret(&response).expect_err("fail"),
            InterpretationError::ContentWithoutToolCall
        ));
    }

    #[test]
    fn test_interpreter_does_not_execute_runtime() {
        let response = make_response(
            "execute_process",
            json!({
                "executable": "cmd.exe",
                "args": ["/c", "echo interpreter_must_not_execute"]
            }),
        );
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        assert_eq!(action.action_type, ActionType::Execute);
        assert_eq!(action.parameters.get("executable").unwrap(), "cmd.exe");
    }

    #[test]
    fn test_read_file_produces_observe_action_type() {
        let response = make_response("read_file", json!({ "path": "Cargo.toml" }));
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        assert_eq!(action.action_type, ActionType::Observe);
        assert_eq!(action.parameters.get("op").unwrap(), "read_file");
    }

    #[test]
    fn test_tool_call_id_becomes_action_id() {
        let response = ProviderResponse {
            content: String::new(),
            tool_calls: vec![ProviderToolCall {
                id: "unique-tool-call-id-42".to_string(),
                name: "read_file".to_string(),
                arguments: json!({ "path": "foo.txt" }),
            }],
            finish_reason: None,
        };
        let decision = interpret(&response).expect("should succeed");
        let action = extract_action(decision);
        assert_eq!(action.id, "unique-tool-call-id-42");
    }
}
