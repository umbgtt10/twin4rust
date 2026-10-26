// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use tempfile::TempDir;
use twin4rust_cli::invocation::entry_point::EntryPoint;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

fn write_file(root: &Path, relative_path: &str, contents: &str) {
    let full_path = root.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(full_path, contents).expect("failed to write file");
}

#[test]
fn run_against_a_manifest_that_does_not_exist_returns_an_error() {
    // Arrange
    let manifest = PathBuf::from("no").join("such").join("Cargo.toml");
    let args = argv(&[
        "cargo-twin4rust",
        "--manifest-path",
        &manifest.to_string_lossy(),
    ]);

    // Act
    let result = EntryPoint::new().run(args);

    // Assert
    assert!(result.is_err());
}

#[test]
fn run_against_this_package_finds_no_missing_mirror() {
    // Arrange
    let args = argv(&[
        "cargo-twin4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        "cargo-twin4rust",
    ]);

    // Act
    let result = EntryPoint::new().run(args);

    // Assert
    assert_eq!(result.expect("run"), ExitCode::SUCCESS);
}

// The exit-code contract, end to end: 1 when a source file has no mirror,
// through the argv cargo hands a subcommand.
#[test]
fn run_through_cargo_against_a_package_missing_a_mirror_exits_with_one() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"gappy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write_file(root, "src/lib.rs", "pub mod engine;\n");
    write_file(root, "src/engine.rs", "pub fn start() -> u32 {\n    1\n}\n");
    let manifest = root.join("Cargo.toml");
    let args = argv(&[
        "cargo-twin4rust",
        "twin4rust",
        "--manifest-path",
        &manifest.to_string_lossy(),
    ]);

    // Act
    let code = EntryPoint::new().run(args).expect("run");

    // Assert
    assert_eq!(code, ExitCode::from(1));
}
