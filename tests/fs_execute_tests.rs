use ai_agent_v3::runtime::{
    append_to_file, create_directory, delete_path, write_file, FilesystemError,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn make_sandbox() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();
    (dir, root)
}

#[test]
fn write_new_file_success() {
    let (_dir, root) = make_sandbox();
    write_file(&root, Path::new("new.txt"), b"data").unwrap();
    assert_eq!(fs::read(root.join("new.txt")).unwrap(), b"data");
}

#[test]
fn append_existing_file_success() {
    let (_dir, root) = make_sandbox();
    let file = root.join("app.txt");
    fs::write(&file, b"init").unwrap();
    append_to_file(&root, Path::new("app.txt"), b"_append").unwrap();
    assert_eq!(fs::read(file).unwrap(), b"init_append");
}

#[test]
fn create_directory_nested_success() {
    let (_dir, root) = make_sandbox();
    create_directory(&root, Path::new("a/b/c")).unwrap();
    assert!(root.join("a/b/c").is_dir());
}

#[test]
fn delete_regular_file_success() {
    let (_dir, root) = make_sandbox();
    let file = root.join("del.txt");
    fs::write(&file, b"x").unwrap();
    delete_path(&root, Path::new("del.txt")).unwrap();
    assert!(!file.exists());
}

#[test]
fn delete_sandbox_root_fails() {
    let (_dir, root) = make_sandbox();
    let err = delete_path(&root, Path::new(".")).unwrap_err();
    assert!(matches!(err, FilesystemError::PolicyViolation { .. }));
}

#[test]
fn write_to_directory_fails() {
    let (_dir, root) = make_sandbox();
    let dir = root.join("subdir");
    fs::create_dir(&dir).unwrap();
    let err = write_file(&root, Path::new("subdir"), b"x").unwrap_err();
    assert!(matches!(err, FilesystemError::IsDirectory(_)));
}

#[test]
fn create_existing_file_as_dir_fails() {
    let (_dir, root) = make_sandbox();
    let file = root.join("f.txt");
    fs::write(&file, b"x").unwrap();
    // Path safety might reject nested components or not, but 'a/f.txt' should be rejected.
    // The requirement says: existing file -> AlreadyExists.
    let err = create_directory(&root, Path::new("f.txt")).unwrap_err();
    assert!(matches!(err, FilesystemError::AlreadyExists(_)));
}

#[test]
fn delete_missing_target_fails() {
    let (_dir, root) = make_sandbox();
    let err = delete_path(&root, Path::new("missing")).unwrap_err();
    assert!(matches!(err, FilesystemError::NotFound(_)));
}

#[test]
fn traversal_rejected_all_ops() {
    let (_dir, root) = make_sandbox();
    let bad = Path::new("../outside.txt");
    assert!(matches!(
        write_file(&root, bad, b"x"),
        Err(FilesystemError::PolicyViolation { .. })
    ));
    assert!(matches!(
        append_to_file(&root, bad, b"x"),
        Err(FilesystemError::PolicyViolation { .. })
    ));
    assert!(matches!(
        delete_path(&root, bad),
        Err(FilesystemError::PolicyViolation { .. })
    ));
    assert!(matches!(
        create_directory(&root, bad),
        Err(FilesystemError::PolicyViolation { .. })
    ));
}

#[test]
fn outside_sandbox_rejected_all_ops() {
    let (_dir, root) = make_sandbox();
    let outside = root.parent().unwrap();
    assert!(matches!(
        write_file(&root, outside, b"x"),
        Err(FilesystemError::PolicyViolation { .. })
    ));
    assert!(matches!(
        append_to_file(&root, outside, b"x"),
        Err(FilesystemError::PolicyViolation { .. })
    ));
    assert!(matches!(
        delete_path(&root, outside),
        Err(FilesystemError::PolicyViolation { .. })
    ));
    assert!(matches!(
        create_directory(&root, outside),
        Err(FilesystemError::PolicyViolation { .. })
    ));
}

#[test]
fn nul_path_rejected_all_ops() {
    let (_dir, root) = make_sandbox();
    let bad = PathBuf::from("foo\0bar");
    assert!(matches!(
        write_file(&root, &bad, b"x"),
        Err(FilesystemError::NulByte(_))
    ));
    assert!(matches!(
        append_to_file(&root, &bad, b"x"),
        Err(FilesystemError::NulByte(_))
    ));
    assert!(matches!(
        delete_path(&root, &bad),
        Err(FilesystemError::NulByte(_))
    ));
    assert!(matches!(
        create_directory(&root, &bad),
        Err(FilesystemError::NulByte(_))
    ));
}
