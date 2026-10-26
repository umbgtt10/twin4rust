// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use std::error::Error;
use twin4rust::workspace::workspace_read_failure::WorkspaceReadFailure;

#[test]
fn source_returns_the_cause() {
    // Arrange
    let failure = WorkspaceReadFailure {
        cause: Box::new(NotHeld),
    };

    // Act
    let cause = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(cause, Some(String::from("not held")));
}

#[test]
fn to_string_says_the_metadata_could_not_be_read() {
    // Arrange
    let failure = WorkspaceReadFailure {
        cause: Box::new(NotHeld),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(text, "failed to read Cargo metadata");
}
