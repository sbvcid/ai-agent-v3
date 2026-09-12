//! Runtime capability modules.
//!
//! Each submodule provides a concrete Runtime capability that Agent Core can use
//! via the generic `Action → ActionResult` interface. Runtime modules must not
//! make high-level decisions; they execute, observe, and return structured results.
//!
//! Stage 6 introduces path-safety primitives for the Filesystem Runtime.
//! The full FilesystemRuntime I/O operations are added in subsequent substeps.

pub mod path_safety;

pub use path_safety::{FilesystemError, SandboxedPath};
