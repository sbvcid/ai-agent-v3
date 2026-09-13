use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::core::types::{Action, ValidationError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessSpec {
    pub executable: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: BTreeMap<String, String>,
    pub timeout_ms: Option<u64>,
}

impl ProcessSpec {
    pub fn from_action(action: &Action) -> Result<Self, ValidationError> {
        let executable = action.parameters.get("executable")
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| ValidationError::EmptyField("executable".to_string()))?
            .clone();

        let args = action.parameters.get("args")
            .map(|s| serde_json::from_str::<Vec<String>>(s).map_err(|_| ValidationError::InvariantViolation("Invalid args JSON".to_string())))
            .transpose()?
            .unwrap_or_default();

        let cwd = action.parameters.get("cwd").cloned();
        
        let env = action.parameters.get("env")
            .map(|s| serde_json::from_str::<BTreeMap<String, String>>(s).map_err(|_| ValidationError::InvariantViolation("Invalid env JSON".to_string())))
            .transpose()?
            .unwrap_or_default();

        let timeout_ms = action.parameters.get("timeout_ms")
            .map(|s| s.parse::<u64>().map_err(|_| ValidationError::InvariantViolation("Invalid timeout_ms".to_string())))
            .transpose()?;

        let spec = Self { executable, args, cwd, env, timeout_ms };
        spec.validate_contract()?;
        Ok(spec)
    }

    pub fn validate_contract(&self) -> Result<(), ValidationError> {
        if self.executable.trim().is_empty() {
            return Err(ValidationError::EmptyField("executable".to_string()));
        }
        if self.executable.contains("..") {
            return Err(ValidationError::InvariantViolation("Path traversal in executable".to_string()));
        }
        if self.args.len() > 1000 {
            return Err(ValidationError::InvariantViolation("Too many arguments".to_string()));
        }
        for arg in &self.args {
            if arg.len() > 4096 {
                return Err(ValidationError::InvariantViolation("Argument too long".to_string()));
            }
        }
        Ok(())
    }
}
