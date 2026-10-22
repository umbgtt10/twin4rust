// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use twin4rust::source_walker::SourceWalker;

fn write_file(root: &Path, relative_path: &str, contents: &str) {
    let full_path = root.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(full_path, contents).expect("failed to write file");
}

#[test]
fn walk_empty_directory_returns_empty() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    // Act
    let files = SourceWalker::walk(root);

    // Assert
    assert!(files.is_empty());
}

#[test]
fn walk_ignores_non_rs_files() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root, "readme.md", "# Readme");
    write_file(root, "config.json", "{}");
    write_file(root, "script.py", "print('hello')");

    // Act
    let files = SourceWalker::walk(root);

    // Assert
    assert!(files.is_empty());
}

#[test]
fn walk_mixed_files_returns_only_rs_files() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root, "src/lib.rs", "pub fn lib() {}");
    write_file(root, "src/main.rs", "fn main() {}");
    write_file(root, "readme.md", "# Readme");
    write_file(root, "tests/test.rs", "#[test] fn t() {}");

    // Act
    let files = SourceWalker::walk(root);

    // Assert
    assert_eq!(files.len(), 3);
    for file in &files {
        let ext = file.extension().unwrap().to_string_lossy().to_string();
        assert_eq!(ext, "rs");
    }
}

#[test]
fn walk_nested_directories_returns_all_rs_files_recursively() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root, "src/lib.rs", "");
    write_file(root, "src/foo/mod.rs", "");
    write_file(root, "src/foo/bar.rs", "");
    write_file(root, "src/baz/qux.rs", "");

    // Act
    let files = SourceWalker::walk(root);

    // Assert
    assert_eq!(files.len(), 4);
}

#[test]
fn walk_non_existent_directory_returns_empty() {
    // Arrange
    let root = PathBuf::from("C:\\non_existent_path_for_testing_12345");

    // Act
    let files = SourceWalker::walk(&root);

    // Assert
    assert!(files.is_empty());
}

#[test]
fn walk_single_rs_file_returns_that_file() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root, "src/lib.rs", "pub fn foo() {}");

    // Act
    let files = SourceWalker::walk(root);

    // Assert
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("lib.rs"));
}
