// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Mirrors src/config.rs.

use std::path::Path;
use std::path::PathBuf;
use twin4rust::config::Config;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let config = Config::new(
        Some(PathBuf::from("Cargo.toml")),
        vec![String::from("node")],
    );

    // Assert
    assert_eq!(config.manifest_path(), Some(Path::new("Cargo.toml")));
    assert_eq!(config.packages(), [String::from("node")]);
}
