use crate::core::process::ProcessSpec;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProcessPolicyError {
    #[error("Executable '{0}' is not allowed by policy")]
    DeniedExecutable(String),
    #[error("Working directory '{0}' is outside the allowed root '{1}'")]
    DeniedWorkingDirectory(String, String),
    #[error("Invalid working directory path")]
    InvalidWorkingDirectory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessPolicy {
    allowed_executables: BTreeSet<String>,
    allowed_cwd_root: Option<PathBuf>,
}

impl Default for ProcessPolicy {
    fn default() -> Self {
        let mut executables = BTreeSet::new();
        executables.insert("cmd.exe".to_string());
        executables.insert("cmd".to_string());
        Self {
            allowed_executables: executables,
            allowed_cwd_root: None,
        }
    }
}

impl ProcessPolicy {
    pub fn new(allowed_executables: BTreeSet<String>, allowed_cwd_root: Option<PathBuf>) -> Self {
        Self {
            allowed_executables,
            allowed_cwd_root,
        }
    }

    pub fn check(&self, spec: &ProcessSpec) -> Result<(), ProcessPolicyError> {
        let exe_lower = spec.executable.to_lowercase();
        let allowed = self.allowed_executables.iter().any(|allowed| {
            allowed.to_lowercase() == exe_lower
                || Path::new(&spec.executable)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase() == allowed.to_lowercase())
                    .unwrap_or(false)
        });

        if !allowed {
            return Err(ProcessPolicyError::DeniedExecutable(
                spec.executable.clone(),
            ));
        }

        if let Some(ref root) = self.allowed_cwd_root {
            if let Some(ref cwd_str) = spec.cwd {
                let cwd_path = Path::new(cwd_str);
                let canonical_root = match root.canonicalize() {
                    Ok(p) => p,
                    Err(_) => root.clone(),
                };
                let canonical_cwd = match cwd_path.canonicalize() {
                    Ok(p) => p,
                    Err(_) => cwd_path.to_path_buf(),
                };

                if !is_subpath(&canonical_root, &canonical_cwd) {
                    return Err(ProcessPolicyError::DeniedWorkingDirectory(
                        cwd_str.clone(),
                        root.to_string_lossy().to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

fn is_subpath(base: &Path, sub: &Path) -> bool {
    let base_comps: Vec<_> = base.components().collect();
    let sub_comps: Vec<_> = sub.components().collect();

    if sub_comps.len() < base_comps.len() {
        return false;
    }

    for (b, s) in base_comps.iter().zip(sub_comps.iter()) {
        if b != s {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_policy_allowed_executable() {
        let policy = ProcessPolicy::default();
        let spec = ProcessSpec {
            executable: "cmd.exe".to_string(),
            args: vec![],
            cwd: None,
            env: BTreeMap::new(),
            timeout_ms: None,
        };
        assert!(policy.check(&spec).is_ok());
    }

    #[test]
    fn test_policy_denied_executable() {
        let policy = ProcessPolicy::default();
        let spec = ProcessSpec {
            executable: "powershell.exe".to_string(),
            args: vec![],
            cwd: None,
            env: BTreeMap::new(),
            timeout_ms: None,
        };
        let err = policy.check(&spec).unwrap_err();
        assert!(matches!(err, ProcessPolicyError::DeniedExecutable(_)));
    }

    #[test]
    fn test_policy_cwd_allowed() {
        let root = PathBuf::from("C:\\agent\\workspace");
        let policy = ProcessPolicy::new(
            ["cmd.exe".to_string()].into_iter().collect(),
            Some(root.clone()),
        );
        let spec = ProcessSpec {
            executable: "cmd.exe".to_string(),
            args: vec![],
            cwd: Some("C:\\agent\\workspace\\project".to_string()),
            env: BTreeMap::new(),
            timeout_ms: None,
        };
        assert!(policy.check(&spec).is_ok());
    }

    #[test]
    fn test_policy_cwd_prefix_attack_denied() {
        let root = PathBuf::from("C:\\agent\\workspace");
        let policy = ProcessPolicy::new(
            ["cmd.exe".to_string()].into_iter().collect(),
            Some(root.clone()),
        );
        let spec = ProcessSpec {
            executable: "cmd.exe".to_string(),
            args: vec![],
            cwd: Some("C:\\agent\\workspace-evil".to_string()),
            env: BTreeMap::new(),
            timeout_ms: None,
        };
        let err = policy.check(&spec).unwrap_err();
        assert!(matches!(
            err,
            ProcessPolicyError::DeniedWorkingDirectory(_, _)
        ));
    }
}
