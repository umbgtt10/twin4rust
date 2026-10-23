// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Mirrors src/package_context.rs.

use std::path::Path;
use std::path::PathBuf;
use twin4rust::package_context::PackageContext;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let context = PackageContext::new(
        String::from("node"),
        PathBuf::from("node"),
        vec![PathBuf::from("node/src")],
    );

    // Assert
    assert_eq!(context.name(), "node");
    assert_eq!(context.manifest_dir(), Path::new("node"));
    assert_eq!(context.source_roots(), [PathBuf::from("node/src")]);
}
