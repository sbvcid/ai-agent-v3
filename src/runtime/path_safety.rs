//! Filesystem path-safety primitives for Stage 6.
//!
//! Provides a single canonical entry point [`resolve_path`] that resolves any
//! caller-supplied path against a configured sandbox root and verifies containment
//! before returning a trusted [`SandboxedPath`].
//!
//! # Safety model
//!
//! * **NUL rejection**: any path string containing a NUL byte (`\0`) is rejected
//!   immediately before any filesystem access.
//! * **Existing target**: the full target path is canonicalized via
//!   [`std::fs::canonicalize`], then checked for containment within the sandbox.
//! * **Non-existing target**: the nearest existing ancestor is canonicalized;
//!   the remaining non-existing components are appended; the result is checked for
//!   containment. This avoids the requirement that `canonicalize` needs the path to
//!   already exist.
//! * **Containment check**: uses `Path::starts_with` on canonicalized components,
//!   not a naive byte/string prefix, so `sandbox/foo` never accidentally matches
//!   `sandbox/foobar`.
//! * **No silent normalization**: a path that escapes the sandbox is always
//!   rejected with [`FilesystemError::PolicyViolation`], never silently redirected.
//! * **Configurable sandbox**: the root is a caller-supplied `&Path`; no
//!   hard-coded project-specific paths exist here.
//! * **No Windows privilege assumptions**: no UAC, no elevated-token checks.
//!
//! # Checkpoint ID safety
//!
//! [`validate_checkpoint_id`] accepts only `[A-Za-z0-9_-]`; it rejects `/`, `\`,
//! `..`, NUL, whitespace, and every other character outside that set.

use std::path::{Component, Path, PathBuf};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by path-safety operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FilesystemError {
    /// The path contains a NUL byte and cannot be used safely.
    #[error("Path contains a NUL byte: {0:?}")]
    NulByte(String),

    /// The path resolves to a location outside the configured sandbox root.
    #[error("Path escapes the sandbox: resolved {resolved:?} is outside root {root:?}")]
    PolicyViolation { root: PathBuf, resolved: PathBuf },

    /// A path component or argument is structurally invalid.
    #[error("Invalid path argument: {0}")]
    InvalidArgument(String),

    /// An I/O error occurred while canonicalizing an existing ancestor.
    /// Stored as a `String` so the type can remain `Clone + PartialEq`.
    #[error("I/O error during path resolution: {0}")]
    Io(String),
}

impl From<std::io::Error> for FilesystemError {
    fn from(e: std::io::Error) -> Self {
        FilesystemError::Io(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// SandboxedPath — a verified, trusted path
// ---------------------------------------------------------------------------

/// A path that has been verified to lie within the configured sandbox root.
///
/// Instances can only be produced by [`resolve_path`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxedPath(PathBuf);

impl SandboxedPath {
    /// Borrow the inner path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Consume the wrapper and return the inner `PathBuf`.
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl std::fmt::Display for SandboxedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

// ---------------------------------------------------------------------------
// checkpoint_id validation
// ---------------------------------------------------------------------------

/// Validate that a `checkpoint_id` is safe to embed in a filename.
///
/// Accepts only `[A-Za-z0-9_-]`. Rejects `/`, `\`, `..`, NUL, whitespace,
/// and all other characters. Existing IDs such as `"ckpt-step-1"` pass.
pub fn validate_checkpoint_id(id: &str) -> Result<(), crate::core::types::ValidationError> {
    if id.is_empty() {
        return Err(crate::core::types::ValidationError::EmptyField(
            "checkpoint_id".to_string(),
        ));
    }
    for ch in id.chars() {
        if !matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-') {
            return Err(crate::core::types::ValidationError::InvariantViolation(
                format!(
                    "checkpoint_id contains invalid character {:?}; \
                     only [A-Za-z0-9_-] are permitted",
                    ch
                ),
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// resolve_path
// ---------------------------------------------------------------------------

/// Resolve `input` relative to `sandbox_root`, verifying sandbox containment.
///
/// See the [module-level documentation](self) for the full algorithm.
pub fn resolve_path(sandbox_root: &Path, input: &Path) -> Result<SandboxedPath, FilesystemError> {
    // 1. NUL rejection — check the lossy string representation.
    let input_lossy = input.to_string_lossy();
    if input_lossy.contains('\0') {
        return Err(FilesystemError::NulByte(input_lossy.into_owned()));
    }
    if input_lossy.is_empty() {
        return Err(FilesystemError::InvalidArgument(
            "path must not be empty".to_string(),
        ));
    }

    // 2. Build absolute candidate (relative paths are joined onto sandbox_root).
    let candidate: PathBuf = if input.is_absolute() {
        input.to_path_buf()
    } else {
        sandbox_root.join(input)
    };

    // 3. Lexically normalize (resolve `.` / `..` without filesystem access).
    let normalized = normalize_lexically(&candidate)?;

    // 4. Find the deepest existing ancestor of the normalized path.
    let (existing_ancestor, non_existing_tail) = split_at_existing_ancestor(&normalized)?;

    // 5. Canonicalize the existing ancestor (OS-level symlink resolution).
    let canonical_ancestor = std::fs::canonicalize(&existing_ancestor)?;

    // 6. Re-attach the non-existing tail components.
    let resolved: PathBuf = non_existing_tail
        .iter()
        .fold(canonical_ancestor, |acc, name| acc.join(name));

    // 7. Canonicalize the sandbox root for comparison.
    let canonical_root = canonicalize_root(sandbox_root)?;

    // 8. Component-based containment check.
    if !resolved.starts_with(&canonical_root) {
        return Err(FilesystemError::PolicyViolation {
            root: canonical_root,
            resolved,
        });
    }

    Ok(SandboxedPath(resolved))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Lexically normalize `path` by processing each component:
/// * Strips `.` (CurDir).
/// * Resolves `..` (ParentDir) against the accumulated stack; rejects `..`
///   that would escape above a root or prefix.
/// * Collects Prefix, RootDir, and Normal components as-is.
///
/// Returns an error for paths that attempt to escape above their root.
fn normalize_lexically(path: &Path) -> Result<PathBuf, FilesystemError> {
    // Stack of individual path components as OsStr fragments.
    let mut stack: Vec<std::ffi::OsString> = Vec::new();
    // Track whether we have seen a root/prefix so we know when `..` would escape.
    let mut has_root = false;

    for component in path.components() {
        match component {
            Component::CurDir => {
                // `.` — no-op
            }
            Component::ParentDir => {
                // `..` — pop the last normal component or reject.
                if stack.is_empty() || !has_root {
                    return Err(FilesystemError::InvalidArgument(format!(
                        "path {:?} attempts to escape above its root via '..'",
                        path
                    )));
                }
                // Check whether what we would pop is a root/prefix marker.
                // We push Prefix/Root as the very first element and never pop them.
                if stack.len() == 1 {
                    // Only root/prefix remains — cannot go higher.
                    return Err(FilesystemError::InvalidArgument(format!(
                        "path {:?} attempts to escape above its root via '..'",
                        path
                    )));
                }
                stack.pop();
            }
            Component::Prefix(p) => {
                stack.push(p.as_os_str().to_os_string());
                has_root = true;
            }
            Component::RootDir => {
                // On Unix the only root is `/`; on Windows it follows the prefix.
                if let Some(last) = stack.last_mut() {
                    last.push(component.as_os_str());
                } else {
                    stack.push(component.as_os_str().to_os_string());
                }
                has_root = true;
            }
            Component::Normal(name) => {
                stack.push(name.to_os_string());
            }
        }
    }

    if stack.is_empty() {
        return Err(FilesystemError::InvalidArgument(
            "path normalized to empty".to_string(),
        ));
    }

    // Reconstruct from stack.
    let mut result = PathBuf::from(stack.remove(0));
    for part in stack {
        result.push(part);
    }
    Ok(result)
}

/// Walk `path` toward the root until we find an ancestor that exists on disk.
///
/// Returns `(existing_ancestor, non_existing_tail)` where `non_existing_tail`
/// contains the `OsString` components that were stripped (in **top-down order**,
/// ready to be re-joined).
fn split_at_existing_ancestor(
    path: &Path,
) -> Result<(PathBuf, Vec<std::ffi::OsString>), FilesystemError> {
    let mut current = path.to_path_buf();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();

    loop {
        if current.exists() {
            tail.reverse(); // restore top-down order
            return Ok((current, tail));
        }

        let name = match current.file_name() {
            Some(n) => n.to_os_string(),
            None => {
                // Reached a root or prefix that doesn't exist — unusual on Windows.
                return Err(FilesystemError::InvalidArgument(format!(
                    "no existing ancestor found for path: {:?}",
                    path
                )));
            }
        };

        tail.push(name);
        current = match current.parent() {
            Some(p) => p.to_path_buf(),
            None => {
                return Err(FilesystemError::InvalidArgument(format!(
                    "no existing ancestor found for path: {:?}",
                    path
                )));
            }
        };
    }
}

/// Canonicalize the sandbox root, using the existing-ancestor approach when
/// parts of the root path do not yet exist.
fn canonicalize_root(root: &Path) -> Result<PathBuf, FilesystemError> {
    if root.exists() {
        Ok(std::fs::canonicalize(root)?)
    } else {
        let (existing_ancestor, tail) = split_at_existing_ancestor(root)?;
        let canonical = std::fs::canonicalize(&existing_ancestor)?;
        let result = tail.iter().fold(canonical, |acc, name| acc.join(name));
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_sandbox() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        (dir, root)
    }

    // --- validate_checkpoint_id ---

    #[test]
    fn ckpt_id_valid_passes() {
        assert!(validate_checkpoint_id("ckpt-step-1").is_ok());
        assert!(validate_checkpoint_id("ckpt-step-2").is_ok());
        assert!(validate_checkpoint_id("ABC_XYZ-0").is_ok());
        assert!(validate_checkpoint_id("a").is_ok());
    }

    #[test]
    fn ckpt_id_empty_fails() {
        let e = validate_checkpoint_id("").unwrap_err();
        assert!(matches!(
            e,
            crate::core::types::ValidationError::EmptyField(_)
        ));
    }

    #[test]
    fn ckpt_id_dotdot_fails() {
        let e = validate_checkpoint_id("../evil").unwrap_err();
        assert!(matches!(
            e,
            crate::core::types::ValidationError::InvariantViolation(_)
        ));
    }

    #[test]
    fn ckpt_id_forward_slash_fails() {
        let e = validate_checkpoint_id("foo/bar").unwrap_err();
        assert!(matches!(
            e,
            crate::core::types::ValidationError::InvariantViolation(_)
        ));
    }

    #[test]
    fn ckpt_id_backslash_fails() {
        let e = validate_checkpoint_id("foo\\bar").unwrap_err();
        assert!(matches!(
            e,
            crate::core::types::ValidationError::InvariantViolation(_)
        ));
    }

    #[test]
    fn ckpt_id_nul_fails() {
        let e = validate_checkpoint_id("foo\0bar").unwrap_err();
        assert!(matches!(
            e,
            crate::core::types::ValidationError::InvariantViolation(_)
        ));
    }

    // --- resolve_path: NUL rejection ---

    #[test]
    fn resolve_nul_byte_rejected() {
        let (_dir, root) = make_sandbox();
        let bad = PathBuf::from("foo\0bar");
        let e = resolve_path(&root, &bad).unwrap_err();
        assert!(matches!(e, FilesystemError::NulByte(_)));
    }

    // --- resolve_path: existing path inside sandbox ---

    #[test]
    fn resolve_existing_inside_sandbox_ok() {
        let (_dir, root) = make_sandbox();
        let file = root.join("inside.txt");
        fs::write(&file, b"x").unwrap();
        let sp = resolve_path(&root, Path::new("inside.txt")).unwrap();
        assert_eq!(sp.as_path(), fs::canonicalize(&file).unwrap().as_path());
    }

    // --- resolve_path: existing path outside sandbox rejected ---

    #[test]
    fn resolve_existing_outside_sandbox_rejected() {
        let (_dir, root) = make_sandbox();
        let outside = root.parent().expect("sandbox has parent");
        let e = resolve_path(&root, outside).unwrap_err();
        assert!(
            matches!(e, FilesystemError::PolicyViolation { .. }),
            "got: {:?}",
            e
        );
    }

    // --- resolve_path: non-existing file inside sandbox ---

    #[test]
    fn resolve_nonexisting_inside_sandbox_ok() {
        let (_dir, root) = make_sandbox();
        let sp = resolve_path(&root, Path::new("new_file.txt")).unwrap();
        let expected = fs::canonicalize(&root).unwrap().join("new_file.txt");
        assert_eq!(sp.as_path(), expected.as_path());
    }

    // --- resolve_path: non-existing file whose parent is outside sandbox ---

    #[test]
    fn resolve_nonexisting_outside_sandbox_rejected() {
        let (_dir, root) = make_sandbox();
        let e = resolve_path(&root, Path::new("../outside.txt")).unwrap_err();
        assert!(
            matches!(e, FilesystemError::PolicyViolation { .. }),
            "got: {:?}",
            e
        );
    }

    // --- resolve_path: traversal via ../../ cannot escape ---

    #[test]
    fn resolve_traversal_cannot_escape() {
        let (_dir, root) = make_sandbox();
        let e = resolve_path(&root, Path::new("../../outside")).unwrap_err();
        assert!(
            matches!(e, FilesystemError::PolicyViolation { .. }),
            "got: {:?}",
            e
        );
    }

    // --- resolve_path: absolute path outside sandbox rejected ---

    #[test]
    fn resolve_absolute_outside_sandbox_rejected() {
        let (_dir, root) = make_sandbox();
        let outside_abs = root.parent().expect("sandbox has parent").to_path_buf();
        let e = resolve_path(&root, &outside_abs).unwrap_err();
        assert!(
            matches!(e, FilesystemError::PolicyViolation { .. }),
            "got: {:?}",
            e
        );
    }

    // --- resolve_path: nested non-existing path inside sandbox ---

    #[test]
    fn resolve_nested_nonexisting_inside_sandbox_ok() {
        let (_dir, root) = make_sandbox();
        let sp = resolve_path(&root, Path::new("subdir/newfile.txt")).unwrap();
        let expected = fs::canonicalize(&root)
            .unwrap()
            .join("subdir")
            .join("newfile.txt");
        assert_eq!(sp.as_path(), expected.as_path());
    }

    // --- SandboxedPath API ---

    #[test]
    fn sandboxed_path_into_path_buf() {
        let (_dir, root) = make_sandbox();
        let file = root.join("t.txt");
        fs::write(&file, b"x").unwrap();
        let sp = resolve_path(&root, Path::new("t.txt")).unwrap();
        let pb = sp.into_path_buf();
        assert!(pb.is_absolute());
    }
}
