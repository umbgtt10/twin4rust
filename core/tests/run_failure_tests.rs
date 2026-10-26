// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use std::error::Error;
use syn::parse_file;
use twin4rust::parse_failure::ParseFailure;
use twin4rust::run_failure::RunFailure;
use twin4rust::workspace::source_path::SourcePath;
use twin4rust::workspace::source_read_failure::SourceReadFailure;
use twin4rust::workspace::workspace_read_failure::WorkspaceReadFailure;

fn chain(failure: &RunFailure) -> Vec<String> {
    let mut texts = vec![failure.to_string()];
    let mut next = failure.source();
    while let Some(cause) = next {
        texts.push(cause.to_string());
        next = cause.source();
    }
    texts
}

#[test]
fn from_a_parse_failure_keeps_its_message_and_its_cause() {
    // Arrange
    let cause = parse_file("fn {").err().expect("not Rust");
    let expected = cause.to_string();

    // Act
    let failure = RunFailure::from(ParseFailure { cause });

    // Assert
    assert_eq!(
        chain(&failure),
        vec![String::from("failed to parse Rust source"), expected]
    );
}

#[test]
fn from_a_source_read_failure_keeps_its_message_and_its_cause() {
    // Arrange
    let read = SourceReadFailure {
        path: SourcePath::new(String::from("/p/src/lib.rs")),
        cause: Box::new(NotHeld),
    };

    // Act
    let failure = RunFailure::from(read);

    // Assert
    assert_eq!(
        chain(&failure),
        vec![
            String::from("failed to read source file /p/src/lib.rs"),
            String::from("not held")
        ]
    );
}

#[test]
fn from_a_workspace_read_failure_keeps_its_message_and_its_cause() {
    // Arrange
    let read = WorkspaceReadFailure {
        cause: Box::new(NotHeld),
    };

    // Act
    let failure = RunFailure::from(read);

    // Assert
    assert_eq!(
        chain(&failure),
        vec![
            String::from("failed to read Cargo metadata"),
            String::from("not held")
        ]
    );
}

#[test]
fn to_string_of_a_missing_package_names_it() {
    // Arrange & Act
    let text = RunFailure::PackageNotFound(String::from("nope")).to_string();

    // Assert
    assert_eq!(text, "package nope was not found in the manifest");
}

#[test]
fn to_string_without_a_parent_directory_says_so() {
    // Arrange & Act
    let text = RunFailure::NoParentDirectory.to_string();

    // Assert
    assert_eq!(text, "package manifest has no parent directory");
}

#[test]
fn to_string_without_a_root_package_asks_for_one() {
    // Arrange & Act
    let text = RunFailure::NoRootPackage.to_string();

    // Assert
    assert_eq!(
        text,
        "manifest contains multiple packages; pass --package <name>"
    );
}
