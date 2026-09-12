//! Runtime capability modules.
//!
//! Each submodule provides a concrete Runtime capability that Agent Core can use
//! via the generic `Action → ActionResult` interface. Runtime modules must not
//! make high-level decisions; they execute, observe, and return structured results.
//!
//! Stage 6 Substep A introduces path-safety primitives.
//! Stage 6 Substep B introduces filesystem Observe operations.

pub mod filesystem_observe;
pub mod path_safety;

pub use filesystem_observe::{
    file_exists, file_metadata, list_dir, read_file, EntryType, FilesystemMetadata, MAX_READ_BYTES,
};
pub use path_safety::{FilesystemError, SandboxedPath};
