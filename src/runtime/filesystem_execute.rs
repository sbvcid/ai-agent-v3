//! Stage 6 Substep C — Filesystem Execute capability.
//!
//! Provides write-oriented operations that Agent Core can use to modify
//! the filesystem:
//!
//! * [`write_file`]          — write content to a file (overwrite)
//! * [`append_to_file`]      — append content to a file
//! * [`delete_path`]         — delete a file or empty directory
//! * [`create_directory`]    — create a directory (iterative)
//!
//! # Safety
//!
//! Every operation passes the caller-supplied path through [`resolve_path`]
//! before any I/O. This enforces sandbox-containment verification.
//! [`delete_path`] additionally prevents deletion of the sandbox root itself.

use crate::runtime::path_safety::{resolve_path, FilesystemError};
use std::fs;
use std::io::Write;
use std::path::Path;

// ---------------------------------------------------------------------------
// write_file
// ---------------------------------------------------------------------------

pub fn write_file(
    sandbox_root: &Path,
    input: &Path,
    content: &[u8],
) -> Result<(), FilesystemError> {
    let safe = resolve_path(sandbox_root, input)?;
    let path = safe.as_path();

    if path.exists() {
        let meta = fs::metadata(path).map_err(|e| FilesystemError::io(path, e))?;
        if meta.is_dir() {
            return Err(FilesystemError::IsDirectory(path.to_path_buf()));
        }
    }

    fs::write(path, content).map_err(|e| FilesystemError::io(path, e))
}

// ---------------------------------------------------------------------------
// append_to_file
// ---------------------------------------------------------------------------

pub fn append_to_file(
    sandbox_root: &Path,
    input: &Path,
    content: &[u8],
) -> Result<(), FilesystemError> {
    let safe = resolve_path(sandbox_root, input)?;
    let path = safe.as_path();

    if path.exists() {
        let meta = fs::metadata(path).map_err(|e| FilesystemError::io(path, e))?;
        if meta.is_dir() {
            return Err(FilesystemError::IsDirectory(path.to_path_buf()));
        }
    }

    fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|e| FilesystemError::io(path, e))?
        .write_all(content)
        .map_err(|e| FilesystemError::io(path, e))
}

// ---------------------------------------------------------------------------
// delete_path
// ---------------------------------------------------------------------------

pub fn delete_path(sandbox_root: &Path, input: &Path) -> Result<(), FilesystemError> {
    let safe = resolve_path(sandbox_root, input)?;
    let path = safe.as_path();

    // Check if it's the root
    let canonical_root =
        fs::canonicalize(sandbox_root).map_err(|e| FilesystemError::io(sandbox_root, e))?;
    if path == canonical_root {
        return Err(FilesystemError::PolicyViolation {
            root: canonical_root,
            resolved: path.to_path_buf(),
        });
    }

    let meta = fs::metadata(path).map_err(|e| FilesystemError::io(path, e))?;
    if meta.is_dir() {
        fs::remove_dir(path).map_err(|e| FilesystemError::io(path, e))
    } else {
        fs::remove_file(path).map_err(|e| FilesystemError::io(path, e))
    }
}

// ---------------------------------------------------------------------------
// create_directory
// ---------------------------------------------------------------------------

pub fn create_directory(sandbox_root: &Path, input: &Path) -> Result<(), FilesystemError> {
    // Iterative creation for security: each step must be resolved & checked.
    let mut current = sandbox_root.to_path_buf();

    // Decompose the relative input path into components
    let components: Vec<_> = input.components().collect();

    for comp in components {
        current.push(comp);

        // Ensure the current intermediate path is safe
        let safe = resolve_path(sandbox_root, &current)?;
        let path = safe.as_path();

        if path.exists() {
            let meta = fs::metadata(path).map_err(|e| FilesystemError::io(path, e))?;
            if !meta.is_dir() {
                return Err(FilesystemError::AlreadyExists(path.to_path_buf()));
            }
        } else {
            fs::create_dir(path).map_err(|e| FilesystemError::io(path, e))?;
        }
    }

    Ok(())
}
