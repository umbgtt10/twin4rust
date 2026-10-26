// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::InMemorySourceTree;
use std::path::PathBuf;
use twin4rust::config::Config;
use twin4rust::runner::Runner;

#[test]
fn run_with_a_manifest_that_does_not_exist_returns_an_error() {
    // Arrange
    let config = Config {
        manifest_path: Some(PathBuf::from("no").join("such").join("Cargo.toml")),
        packages: vec![],
    };
    let tree = InMemorySourceTree::new();

    // Act
    let result = Runner::new(&tree).run(&config);

    // Assert
    assert!(result.is_err());
}

#[test]
fn runner_exists_and_is_send() {
    // Arrange & Act
    fn assert_send<T: Send>() {}
    assert_send::<Runner>();

    // Assert
    // Compile-time check: Runner is constructible and Send
}
