// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Mirrors src/missing_test_gap.rs.

use twin4rust::missing_test_gap::MissingTestGap;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let gap = MissingTestGap::new(
        String::from("node"),
        String::from("src/codec.rs"),
        String::from("tests/codec_tests.rs"),
    );

    // Assert
    assert_eq!(gap.package_name(), "node");
    assert_eq!(gap.relative_source_file(), "src/codec.rs");
    assert_eq!(gap.expected_test_file(), "tests/codec_tests.rs");
}
