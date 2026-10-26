// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use twin4rust::traits::source_tree::SourceTree;
use twin4rust::workspace::source_path::SourcePath;
use twin4rust_cli::system::portable_path::PortablePath;
use twin4rust_cli::system::walkdir_source_tree::WalkdirSourceTree;

// A file name that is not UTF-8 can only be made on Linux: Windows names are
// UTF-16, and macOS's file system refuses a name that is not valid UTF-8.
#[cfg(target_os = "linux")]
mod not_utf8 {
    use super::source_path;
    use std::ffi::OsStr;
    use std::fs;
    use std::os::unix::ffi::OsStrExt;
    use tempfile::TempDir;
    use twin4rust::traits::source_tree::SourceTree;
    use twin4rust_cli::system::walkdir_source_tree::WalkdirSourceTree;

    #[test]
    fn rust_files_with_a_file_name_that_is_not_utf8_fails_naming_it() {
        // Arrange
        let dir = TempDir::new().expect("temp dir");
        let name = OsStr::from_bytes(b"bad\xff.rs");
        fs::write(dir.path().join(name), "pub fn f() {}").expect("write");

        // Act
        let files = WalkdirSourceTree::new().rust_files(&source_path(dir.path()));

        // Assert
        let failure = files.expect_err("not UTF-8");
        assert!(failure.to_string().ends_with("bad\u{fffd}.rs"), "{failure}");
        assert_eq!(failure.cause.to_string(), "the path is not valid UTF-8");
    }
}

fn source_path(path: &Path) -> SourcePath {
    PortablePath::of(path).expect("a UTF-8 test path")
}

fn write_file(root: &Path, relative_path: &str, contents: &str) {
    let full_path = root.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(full_path, contents).expect("failed to write file");
}

#[test]
fn contains_a_directory_on_disk_returns_true() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    write_file(dir.path(), "tests/engine_tests.rs/keep.txt", "");

    // Act
    let held = WalkdirSourceTree::new().contains(&source_path(
        &dir.path().join("tests").join("engine_tests.rs"),
    ));

    // Assert
    assert!(held);
}

#[test]
fn contains_a_file_on_disk_returns_true() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    write_file(dir.path(), "tests/engine_tests.rs", "");

    // Act
    let held = WalkdirSourceTree::new().contains(&source_path(
        &dir.path().join("tests").join("engine_tests.rs"),
    ));

    // Assert
    assert!(held);
}

#[test]
fn contains_a_path_that_does_not_exist_returns_false() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");

    // Act
    let held = WalkdirSourceTree::new().contains(&source_path(
        &dir.path().join("tests").join("engine_tests.rs"),
    ));

    // Assert
    assert!(!held);
}

#[test]
fn read_source_of_a_file_on_disk_returns_its_contents() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    write_file(dir.path(), "src/lib.rs", "pub fn foo() {}");

    // Act
    let source =
        WalkdirSourceTree::new().read_source(&source_path(&dir.path().join("src").join("lib.rs")));

    // Assert
    assert_eq!(source.expect("readable"), "pub fn foo() {}");
}

#[test]
fn read_source_of_a_file_that_does_not_exist_returns_an_error() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");

    // Act
    let source =
        WalkdirSourceTree::new().read_source(&source_path(&dir.path().join("src").join("lib.rs")));

    // Assert
    assert!(source.is_err());
}

#[test]
fn rust_files_empty_directory_returns_empty() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    // Act
    let files = WalkdirSourceTree
        .rust_files(&source_path(root))
        .expect("listable");

    // Assert
    assert!(files.is_empty());
}

#[test]
fn rust_files_ignores_non_rs_files() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root, "readme.md", "# Readme");
    write_file(root, "config.json", "{}");
    write_file(root, "script.py", "print('hello')");

    // Act
    let files = WalkdirSourceTree
        .rust_files(&source_path(root))
        .expect("listable");

    // Assert
    assert!(files.is_empty());
}

#[test]
fn rust_files_mixed_files_returns_only_rs_files() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root, "src/lib.rs", "pub fn lib() {}");
    write_file(root, "src/main.rs", "fn main() {}");
    write_file(root, "readme.md", "# Readme");
    write_file(root, "tests/test.rs", "#[test] fn t() {}");

    // Act
    let files = WalkdirSourceTree
        .rust_files(&source_path(root))
        .expect("listable");

    // Assert
    assert_eq!(files.len(), 3);
    for file in &files {
        let ext = file.extension().unwrap().to_string();
        assert_eq!(ext, "rs");
    }
}

#[test]
fn rust_files_nested_directories_returns_all_rs_files_recursively() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root, "src/lib.rs", "");
    write_file(root, "src/foo/mod.rs", "");
    write_file(root, "src/foo/bar.rs", "");
    write_file(root, "src/baz/qux.rs", "");

    // Act
    let files = WalkdirSourceTree
        .rust_files(&source_path(root))
        .expect("listable");

    // Assert
    assert_eq!(files.len(), 4);
}

#[test]
fn rust_files_non_existent_directory_returns_empty() {
    // Arrange
    let root = PathBuf::from("C:\\non_existent_path_for_testing_12345");

    // Act
    let files = WalkdirSourceTree
        .rust_files(&source_path(&root))
        .expect("listable");

    // Assert
    assert!(files.is_empty());
}

#[test]
fn rust_files_single_rs_file_returns_that_file() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root, "src/lib.rs", "pub fn foo() {}");

    // Act
    let files = WalkdirSourceTree
        .rust_files(&source_path(root))
        .expect("listable");

    // Assert
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name(), Some("lib.rs"));
}
