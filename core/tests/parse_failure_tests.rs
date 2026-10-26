// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::error::Error;
use syn::parse_file;
use twin4rust::parse_failure::ParseFailure;

fn failure() -> ParseFailure {
    ParseFailure {
        cause: parse_file("fn {").err().expect("not Rust"),
    }
}

#[test]
fn source_returns_syns_reason() {
    // Arrange
    let failure = failure();

    // Act
    let cause = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(cause, Some(failure.cause.to_string()));
}

#[test]
fn to_string_says_the_source_did_not_parse() {
    // Arrange & Act
    let text = failure().to_string();

    // Assert
    assert_eq!(text, "failed to parse Rust source");
}
