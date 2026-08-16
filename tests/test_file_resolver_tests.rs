// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use twin4rust::test_file_resolver::TestFileResolver;

fn resolver() -> TestFileResolver {
    TestFileResolver::new()
}

fn manifest_dir() -> PathBuf {
    PathBuf::from("/home/user/project")
}

#[test]
fn lib_file_returns_none() {
    // Arrange
    let manifest = manifest_dir();
    let source = manifest.join("src/lib.rs");

    // Act
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert!(result.is_none());
}

#[test]
fn main_file_returns_none() {
    // Arrange
    let manifest = manifest_dir();
    let source = manifest.join("src/main.rs");

    // Act
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert!(result.is_none());
}

#[test]
fn mod_file_returns_none() {
    // Arrange
    let manifest = manifest_dir();
    let source = manifest.join("src/feature/mod.rs");

    // Act
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert!(result.is_none());
}

#[test]
fn flat_source_file_maps_to_tests_root() {
    // Arrange
    let manifest = manifest_dir();
    let source = manifest.join("src/helper.rs");

    // Act
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert_eq!(result, Some(manifest.join("tests/helper_tests.rs")));
}

#[test]
fn nested_source_file_preserves_directory() {
    // Arrange
    let manifest = manifest_dir();
    let source = manifest.join("src/implementations/raft/client.rs");

    // Act
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert_eq!(
        result,
        Some(manifest.join("tests/implementations/raft/client_tests.rs"))
    );
}

#[test]
fn deeply_nested_source_preserves_full_path() {
    // Arrange
    let manifest = manifest_dir();
    let source = manifest.join("src/a/b/c/d/e.rs");

    // Act
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert_eq!(result, Some(manifest.join("tests/a/b/c/d/e_tests.rs")));
}

#[test]
fn source_outside_src_returns_none() {
    // Arrange
    let manifest = manifest_dir();
    let source = manifest.join("build.rs");

    // Act
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert!(result.is_none());
}

#[test]
fn source_in_other_directory_returns_none() {
    // Arrange
    let manifest = manifest_dir();
    let source = PathBuf::from("/tmp/other.rs");

    // Act
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert!(result.is_none());
}

#[test]
fn source_file_with_double_extension_maps_to_tests() {
    // Arrange
    let manifest = manifest_dir();
    let source = manifest.join("src/data.rs.bak");

    // Act — file_stem() returns "data.rs"
    let result = resolver().expected_test_file(&manifest, &source);

    // Assert
    assert_eq!(result, Some(manifest.join("tests/data.rs_tests.rs")));
}
