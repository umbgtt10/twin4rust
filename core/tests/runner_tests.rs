// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::InMemorySourceTree;
use crate::support::in_memory_workspace_reader::InMemoryWorkspaceReader;
use twin4rust::config::Config;
use twin4rust::runner::Runner;

fn config() -> Config {
    Config {
        packages: vec![String::from("alpha"), String::from("beta")],
    }
}

#[test]
fn run_with_a_workspace_that_cannot_be_read_returns_the_error() {
    // Arrange
    let reader = InMemoryWorkspaceReader::failing();
    let tree = InMemorySourceTree::new();

    // Act
    let result = Runner::new(&reader, &tree).run(&config());

    // Assert
    assert_eq!(
        result.err().map(|failure| failure.to_string()),
        Some(String::from("failed to read Cargo metadata"))
    );
}

#[test]
fn runner_exists_and_is_send() {
    // Arrange & Act
    fn assert_send<T: Send>() {}
    assert_send::<Runner>();

    // Assert
    // Compile-time check: Runner is constructible and Send
}
