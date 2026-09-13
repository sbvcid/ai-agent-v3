//! Stage 6 Substep D — Filesystem Runtime capability adapter.
//!
//! Provides [`FilesystemRuntime`], which implements the Core [`Runtime`] trait
//! by dispatching canonical [`Action`] requests to the concrete filesystem
//! operations defined in Stage 6A/6B/6C.

use crate::core::runtime::Runtime;
use crate::core::types::{Action, ActionResult, ActionType};
use crate::runtime::filesystem_execute::{
    append_to_file, create_directory, delete_path, write_file,
};
use crate::runtime::filesystem_observe::{
    file_exists, file_metadata, list_dir, read_file,
};
use std::path::{Path, PathBuf};

/// Filesystem Runtime implementation that executes canonical Actions
/// against a configured sandbox root directory.
#[derive(Debug, Clone)]
pub struct FilesystemRuntime {
    sandbox_root: PathBuf,
}

impl FilesystemRuntime {
    /// Create a new FilesystemRuntime bound to the specified sandbox root directory.
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self {
            sandbox_root: sandbox_root.into(),
        }
    }

    /// Immutable reference to the configured sandbox root.
    pub fn sandbox_root(&self) -> &Path {
        &self.sandbox_root
    }
}

impl Runtime for FilesystemRuntime {
    fn execute(&mut self, action: Action) -> ActionResult {
        let op = match action.parameters.get("op") {
            Some(op) if !op.trim().is_empty() => op.as_str(),
            _ => {
                return ActionResult::failure(
                    &action.id,
                    "Missing or empty required parameter 'op'",
                );
            }
        };

        match action.action_type {
            ActionType::Observe => match op {
                "read_file" => {
                    let path_str = match action.parameters.get("path") {
                        Some(p) => p,
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'path' for read_file",
                            );
                        }
                    };
                    match read_file(&self.sandbox_root, Path::new(path_str)) {
                        Ok(content) => {
                            let mut res = ActionResult::success(&action.id, content);
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                        Err(e) => {
                            let mut res = ActionResult::failure(&action.id, e.to_string());
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                    }
                }
                "file_exists" => {
                    let path_str = match action.parameters.get("path") {
                        Some(p) => p,
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'path' for file_exists",
                            );
                        }
                    };
                    match file_exists(&self.sandbox_root, Path::new(path_str)) {
                        Ok(exists) => {
                            let mut res = ActionResult::success(
                                &action.id,
                                if exists { "true" } else { "false" },
                            );
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                        Err(e) => {
                            let mut res = ActionResult::failure(&action.id, e.to_string());
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                    }
                }
                "list_dir" => {
                    let path_str = match action.parameters.get("path") {
                        Some(p) => p,
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'path' for list_dir",
                            );
                        }
                    };
                    match list_dir(&self.sandbox_root, Path::new(path_str)) {
                        Ok(entries) => {
                            let mut res = ActionResult::success(&action.id, entries.join("\n"));
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                        Err(e) => {
                            let mut res = ActionResult::failure(&action.id, e.to_string());
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                    }
                }
                "file_metadata" => {
                    let path_str = match action.parameters.get("path") {
                        Some(p) => p,
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'path' for file_metadata",
                            );
                        }
                    };
                    match file_metadata(&self.sandbox_root, Path::new(path_str)) {
                        Ok(meta) => {
                            let summary = format!(
                                "type={:?}, size_bytes={}, modified_epoch={:?}",
                                meta.entry_type, meta.size_bytes, meta.modified_secs_epoch
                            );
                            let mut res = ActionResult::success(&action.id, summary);
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                        Err(e) => {
                            let mut res = ActionResult::failure(&action.id, e.to_string());
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                    }
                }
                other => ActionResult::failure(
                    &action.id,
                    format!("Unknown observe operation '{}'", other),
                ),
            },

            ActionType::Execute => match op {
                "write_file" => {
                    let path_str = match action.parameters.get("path") {
                        Some(p) => p,
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'path' for write_file",
                            );
                        }
                    };
                    let content = match action.parameters.get("content") {
                        Some(c) => c.as_bytes(),
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'content' for write_file",
                            );
                        }
                    };
                    match write_file(&self.sandbox_root, Path::new(path_str), content) {
                        Ok(()) => {
                            let mut res =
                                ActionResult::success(&action.id, "File written successfully");
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                        Err(e) => {
                            let mut res = ActionResult::failure(&action.id, e.to_string());
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                    }
                }
                "append_to_file" => {
                    let path_str = match action.parameters.get("path") {
                        Some(p) => p,
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'path' for append_to_file",
                            );
                        }
                    };
                    let content = match action.parameters.get("content") {
                        Some(c) => c.as_bytes(),
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'content' for append_to_file",
                            );
                        }
                    };
                    match append_to_file(&self.sandbox_root, Path::new(path_str), content) {
                        Ok(()) => {
                            let mut res =
                                ActionResult::success(&action.id, "Content appended successfully");
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                        Err(e) => {
                            let mut res = ActionResult::failure(&action.id, e.to_string());
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                    }
                }
                "delete_path" => {
                    let path_str = match action.parameters.get("path") {
                        Some(p) => p,
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'path' for delete_path",
                            );
                        }
                    };
                    match delete_path(&self.sandbox_root, Path::new(path_str)) {
                        Ok(()) => {
                            let mut res =
                                ActionResult::success(&action.id, "Path deleted successfully");
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                        Err(e) => {
                            let mut res = ActionResult::failure(&action.id, e.to_string());
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                    }
                }
                "create_directory" => {
                    let path_str = match action.parameters.get("path") {
                        Some(p) => p,
                        None => {
                            return ActionResult::failure(
                                &action.id,
                                "Missing required parameter 'path' for create_directory",
                            );
                        }
                    };
                    match create_directory(&self.sandbox_root, Path::new(path_str)) {
                        Ok(()) => {
                            let mut res = ActionResult::success(
                                &action.id,
                                "Directory created successfully",
                            );
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                        Err(e) => {
                            let mut res = ActionResult::failure(&action.id, e.to_string());
                            res.affected_resources.push(path_str.clone());
                            res
                        }
                    }
                }
                other => ActionResult::failure(
                    &action.id,
                    format!("Unknown execute operation '{}'", other),
                ),
            },

            ActionType::Interact | ActionType::Wait => ActionResult::failure(
                &action.id,
                format!(
                    "FilesystemRuntime does not support ActionType::{:?}",
                    action.action_type
                ),
            ),
        }
    }
}
