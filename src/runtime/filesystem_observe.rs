//! Stage 6 Substep B — Filesystem Observe capability.
//!
//! Provides four read-only operations that Agent Core can use to gather
//! environmental evidence from the filesystem:
//!
//! * [`read_file`]   — read file contents (bounded, text-oriented)
//! * [`file_exists`] — check whether a path exists (never an error for missing)
//! * [`list_dir`]    — list direct children of a directory (sorted, deterministic)
//! * [`file_metadata`] — minimal metadata (type, size, modification time)
//!
//! # Safety
//!
//! Every operation passes the caller-supplied path through [`resolve_path`]
//! before any I/O. This enforces:
//! * NUL-byte rejection
//! * `..` traversal rejection
//! * sandbox-containment verification
//!
//! Runtime does not decide whether an operation *should* happen; it only
//! executes the requested capability and returns a structured result.
//! Agent Core makes all policy decisions.
//!
//! # Read bound
//!
//! [`read_file`] reads at most [`MAX_READ_BYTES`] bytes. Files larger than this
//! are read up to that limit; the returned content is valid UTF-8 (lossily
//! converted). Callers receive a truncated result rather than an error, so that
//! Agent Core can still act on partial evidence.

use crate::runtime::path_safety::{resolve_path, FilesystemError};
use std::path::Path;

/// Maximum number of bytes [`read_file`] will read in a single call.
///
/// 1 MiB. Files larger than this are truncated to this limit; the caller
/// receives the first `MAX_READ_BYTES` bytes converted to UTF-8.
pub const MAX_READ_BYTES: u64 = 1024 * 1024;

// ---------------------------------------------------------------------------
// Metadata types
// ---------------------------------------------------------------------------

/// The kind of filesystem entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// Anything else (symlink, device, etc.).
    Other,
}

/// Minimal metadata for a filesystem entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemMetadata {
    /// The type of the entry (file, directory, or other).
    pub entry_type: EntryType,
    /// Size in bytes.  For directories this is the size reported by the OS,
    /// which is not necessarily meaningful; callers should not rely on it for
    /// directories.
    pub size_bytes: u64,
    /// Seconds since the Unix epoch of the last modification time, if
    /// available.  Returns `None` when the platform does not support it or
    /// the value is outside the representable range.
    pub modified_secs_epoch: Option<u64>,
}

// ---------------------------------------------------------------------------
// read_file
// ---------------------------------------------------------------------------

/// Read the contents of a file inside the sandbox.
///
/// Returns the file contents as a `String`.  Files are read up to
/// [`MAX_READ_BYTES`]; anything beyond that limit is silently truncated.
/// Non-UTF-8 bytes are replaced with the Unicode replacement character.
///
/// # Errors
///
/// * [`FilesystemError::NulByte`] / [`FilesystemError::PolicyViolation`] /
///   [`FilesystemError::InvalidArgument`] — path-safety failures.
/// * [`FilesystemError::Io`] — the path exists but cannot be read (e.g.
///   permission denied, is a directory, etc.).
pub fn read_file(sandbox_root: &Path, input: &Path) -> Result<String, FilesystemError> {
    let safe = resolve_path(sandbox_root, input)?;
    let path = safe.as_path();

    // Read up to MAX_READ_BYTES.
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut buf = Vec::new();
    reader.by_ref().take(MAX_READ_BYTES).read_to_end(&mut buf)?;

    Ok(String::from_utf8_lossy(&buf).into_owned())
}

// ---------------------------------------------------------------------------
// file_exists
// ---------------------------------------------------------------------------

/// Check whether a path exists inside the sandbox.
///
/// Returns `Ok(true)` if the path exists, `Ok(false)` if it does not.
/// A missing path is **not** an error — it is a deterministic observation.
///
/// # Errors
///
/// * Path-safety failures (NUL, traversal, sandbox escape).
/// * [`FilesystemError::Io`] — the *parent* path could not be inspected
///   due to a permission error during path resolution.
pub fn file_exists(sandbox_root: &Path, input: &Path) -> Result<bool, FilesystemError> {
    let safe = resolve_path(sandbox_root, input)?;
    Ok(safe.as_path().exists())
}

// ---------------------------------------------------------------------------
// list_dir
// ---------------------------------------------------------------------------

/// List the direct children of a directory inside the sandbox.
///
/// Returns a **sorted** `Vec<String>` of entry names (not full paths).
/// The sort is lexicographic on the `OsString` representation, ensuring
/// deterministic output regardless of filesystem iteration order.
///
/// # Errors
///
/// * Path-safety failures.
/// * [`FilesystemError::Io`] — the directory does not exist or cannot be read.
pub fn list_dir(sandbox_root: &Path, input: &Path) -> Result<Vec<String>, FilesystemError> {
    let safe = resolve_path(sandbox_root, input)?;
    let path = safe.as_path();

    let mut entries: Vec<String> = std::fs::read_dir(path)?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?;

    entries.sort();
    Ok(entries)
}

// ---------------------------------------------------------------------------
// file_metadata
// ---------------------------------------------------------------------------

/// Retrieve minimal metadata for a path inside the sandbox.
///
/// # Errors
///
/// * Path-safety failures.
/// * [`FilesystemError::Io`] — the path does not exist or cannot be stat-ed.
pub fn file_metadata(
    sandbox_root: &Path,
    input: &Path,
) -> Result<FilesystemMetadata, FilesystemError> {
    let safe = resolve_path(sandbox_root, input)?;
    let path = safe.as_path();

    let meta = std::fs::metadata(path)?;

    let entry_type = if meta.is_file() {
        EntryType::File
    } else if meta.is_dir() {
        EntryType::Directory
    } else {
        EntryType::Other
    };

    let size_bytes = meta.len();

    let modified_secs_epoch = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    Ok(FilesystemMetadata {
        entry_type,
        size_bytes,
        modified_secs_epoch,
    })
}

// ---------------------------------------------------------------------------
// Unit / integration tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn make_sandbox() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        (dir, root)
    }

    // -----------------------------------------------------------------------
    // read_file
    // -----------------------------------------------------------------------

    #[test]
    fn read_existing_file_returns_contents() {
        let (_dir, root) = make_sandbox();
        let file = root.join("hello.txt");
        fs::write(&file, b"hello world").unwrap();

        let content = read_file(&root, Path::new("hello.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn read_missing_file_returns_io_error() {
        let (_dir, root) = make_sandbox();
        let err = read_file(&root, Path::new("does_not_exist.txt")).unwrap_err();
        // resolve_path succeeds (non-existing inside sandbox), but File::open fails.
        assert!(
            matches!(err, FilesystemError::NotFound(_)),
            "expected NotFound error, got: {:?}",
            err
        );
    }

    #[test]
    fn read_outside_sandbox_rejected() {
        let (_dir, root) = make_sandbox();
        // Parent of sandbox is outside it.
        let outside = root.parent().expect("has parent");
        let err = read_file(&root, outside).unwrap_err();
        assert!(
            matches!(err, FilesystemError::PolicyViolation { .. }),
            "got: {:?}",
            err
        );
    }

    #[test]
    fn read_traversal_rejected() {
        let (_dir, root) = make_sandbox();
        let err = read_file(&root, Path::new("../secret.txt")).unwrap_err();
        assert!(
            matches!(err, FilesystemError::PolicyViolation { .. }),
            "got: {:?}",
            err
        );
    }

    // -----------------------------------------------------------------------
    // file_exists
    // -----------------------------------------------------------------------

    #[test]
    fn exists_existing_file_returns_true() {
        let (_dir, root) = make_sandbox();
        let file = root.join("present.txt");
        fs::write(&file, b"x").unwrap();

        let result = file_exists(&root, Path::new("present.txt")).unwrap();
        assert!(result);
    }

    #[test]
    fn exists_missing_file_returns_false_not_error() {
        let (_dir, root) = make_sandbox();
        let result = file_exists(&root, Path::new("absent.txt")).unwrap();
        assert!(!result);
    }

    // -----------------------------------------------------------------------
    // list_dir
    // -----------------------------------------------------------------------

    #[test]
    fn list_directory_returns_sorted_entries() {
        let (_dir, root) = make_sandbox();
        fs::write(root.join("c.txt"), b"").unwrap();
        fs::write(root.join("a.txt"), b"").unwrap();
        fs::write(root.join("b.txt"), b"").unwrap();

        let entries = list_dir(&root, Path::new(".")).unwrap();
        // Sort is lexicographic; result must be a.txt, b.txt, c.txt.
        assert_eq!(entries, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn list_directory_output_is_deterministic() {
        let (_dir, root) = make_sandbox();
        for name in &["z.txt", "m.txt", "a.txt"] {
            fs::write(root.join(name), b"").unwrap();
        }

        let r1 = list_dir(&root, Path::new(".")).unwrap();
        let r2 = list_dir(&root, Path::new(".")).unwrap();
        assert_eq!(r1, r2);
        assert_eq!(r1, vec!["a.txt", "m.txt", "z.txt"]);
    }

    #[test]
    fn list_missing_directory_returns_io_error() {
        let (_dir, root) = make_sandbox();
        let err = list_dir(&root, Path::new("no_such_dir")).unwrap_err();
        assert!(
            matches!(err, FilesystemError::NotFound(_)),
            "got: {:?}",
            err
        );
    }

    // -----------------------------------------------------------------------
    // file_metadata
    // -----------------------------------------------------------------------

    #[test]
    fn metadata_existing_file_returns_correct_type_and_size() {
        let (_dir, root) = make_sandbox();
        let file = root.join("data.txt");
        fs::write(&file, b"abcde").unwrap();

        let meta = file_metadata(&root, Path::new("data.txt")).unwrap();
        assert_eq!(meta.entry_type, EntryType::File);
        assert_eq!(meta.size_bytes, 5);
    }

    #[test]
    fn metadata_directory_returns_directory_type() {
        let (_dir, root) = make_sandbox();
        let sub = root.join("subdir");
        fs::create_dir(&sub).unwrap();

        let meta = file_metadata(&root, Path::new("subdir")).unwrap();
        assert_eq!(meta.entry_type, EntryType::Directory);
    }

    // -----------------------------------------------------------------------
    // NUL path rejection (goes through resolve_path)
    // -----------------------------------------------------------------------

    #[test]
    fn nul_path_rejected_for_all_observe_ops() {
        let (_dir, root) = make_sandbox();
        let nul = PathBuf::from("foo\0bar");

        assert!(matches!(
            read_file(&root, &nul).unwrap_err(),
            FilesystemError::NulByte(_)
        ));
        assert!(matches!(
            file_exists(&root, &nul).unwrap_err(),
            FilesystemError::NulByte(_)
        ));
        assert!(matches!(
            list_dir(&root, &nul).unwrap_err(),
            FilesystemError::NulByte(_)
        ));
        assert!(matches!(
            file_metadata(&root, &nul).unwrap_err(),
            FilesystemError::NulByte(_)
        ));
    }
}
