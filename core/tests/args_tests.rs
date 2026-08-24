// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::Parser;
use twin4rust::args::Args;

#[test]
fn parse_args_all_options() {
    // Arrange & Act
    let args = Args::parse_from([
        "cargo-twin4rust",
        "--manifest-path",
        "my/Cargo.toml",
        "--package",
        "pkg_a",
        "--package",
        "pkg_b",
    ]);

    // Assert
    assert_eq!(
        args.manifest_path.unwrap().to_string_lossy(),
        "my/Cargo.toml"
    );
    assert_eq!(args.packages, vec!["pkg_a", "pkg_b"]);
}

#[test]
fn parse_args_defaults() {
    // Arrange & Act
    let args = Args::parse_from(["cargo-twin4rust"]);

    // Assert
    assert!(args.manifest_path.is_none());
    assert!(args.packages.is_empty());
}

#[test]
fn parse_args_manifest_path() {
    // Arrange & Act
    let args = Args::parse_from(["cargo-twin4rust", "--manifest-path", "Cargo.toml"]);

    // Assert
    assert_eq!(args.manifest_path.unwrap().to_string_lossy(), "Cargo.toml");
}

#[test]
fn parse_args_multiple_packages() {
    // Arrange & Act
    let args = Args::parse_from([
        "cargo-twin4rust",
        "--package",
        "foo",
        "--package",
        "bar",
        "--package",
        "baz",
    ]);

    // Assert
    assert_eq!(args.packages, vec!["foo", "bar", "baz"]);
}

#[test]
fn parse_args_single_package() {
    // Arrange & Act
    let args = Args::parse_from(["cargo-twin4rust", "--package", "foo"]);

    // Assert
    assert_eq!(args.packages, vec!["foo"]);
}

#[test]
fn without_cargo_subcommand_drops_the_name_cargo_inserts() {
    // Arrange -- `cargo twin4rust --package foo` reaches the binary as
    // `cargo-twin4rust twin4rust --package foo`. Leaving that second argument in
    // place makes clap reject every real invocation through cargo.
    let raw = ["cargo-twin4rust", "twin4rust", "--package", "foo"].map(String::from);

    // Act
    let forwarded = Args::without_cargo_subcommand(raw);

    // Assert
    assert_eq!(forwarded, vec!["cargo-twin4rust", "--package", "foo"]);
}

#[test]
fn without_cargo_subcommand_handles_being_given_nothing() {
    // Arrange -- an empty argv has no element 1 to inspect. Indexing it would
    // panic before the process ever reached clap.
    let raw: [String; 0] = [];

    // Act
    let forwarded = Args::without_cargo_subcommand(raw);

    // Assert
    assert!(forwarded.is_empty());
}

#[test]
fn without_cargo_subcommand_keeps_a_package_that_happens_to_be_named_twin4rust() {
    // Arrange -- the guard looks at position 1 only. A package legitimately
    // called twin4rust appears at position 2, behind its flag, so it must
    // survive. Matching anywhere would silently drop it from the analysis.
    let raw = ["cargo-twin4rust", "--package", "twin4rust"].map(String::from);

    // Act
    let args = Args::parse_from(Args::without_cargo_subcommand(raw));

    // Assert
    assert_eq!(args.packages, vec!["twin4rust"]);
}

#[test]
fn without_cargo_subcommand_leaves_a_direct_invocation_untouched() {
    // Arrange -- run straight from target/release the name is not repeated.
    // Stripping unconditionally would eat the user's first real argument.
    let raw = ["cargo-twin4rust", "--package", "foo"].map(String::from);

    // Act
    let forwarded = Args::without_cargo_subcommand(raw);

    // Assert
    assert_eq!(forwarded, vec!["cargo-twin4rust", "--package", "foo"]);
}
