// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use twin4rust::args::Args;
use twin4rust::runner::Runner;

#[test]
fn run_against_a_manifest_that_does_not_exist_returns_an_error() {
    // Arrange
    let args = Args {
        manifest_path: Some(PathBuf::from("no").join("such").join("Cargo.toml")),
        packages: vec![],
    };

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

#[test]
fn run_against_this_package_finds_no_missing_mirror() {
    // Arrange
    let args = Args {
        manifest_path: Some(PathBuf::from("Cargo.toml")),
        packages: vec![String::from("cargo-twin4rust")],
    };

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn runner_exists_and_is_send() {
    // Arrange & Act
    fn assert_send<T: Send>() {}
    assert_send::<Runner>();

    // Assert
    // Compile-time check: Runner is constructible and Send
}
