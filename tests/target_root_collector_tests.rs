// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use cargo_metadata::Target;
use serde_json::from_value;
use std::path::PathBuf;
use twin4rust::target_root_collector::TargetRootCollector;

fn bench_target(src_path: &str) -> Target {
    from_value(serde_json::json!({
        "name": "bench_heavy",
        "kind": ["bench"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": false,
        "doctest": false,
        "test": false
    }))
    .expect("valid Target")
}

fn bin_target(src_path: &str) -> Target {
    from_value(serde_json::json!({
        "name": "my_bin",
        "kind": ["bin"],
        "crate_types": ["bin"],
        "src_path": src_path,
        "edition": "2021",
        "doc": true,
        "doctest": false,
        "test": true
    }))
    .expect("valid Target")
}

fn custom_build_target(src_path: &str) -> Target {
    from_value(serde_json::json!({
        "name": "build_script",
        "kind": ["custom-build"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": false,
        "doctest": false,
        "test": false
    }))
    .expect("valid Target")
}

fn example_target(src_path: &str) -> Target {
    from_value(serde_json::json!({
        "name": "example_demo",
        "kind": ["example"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": false,
        "doctest": false,
        "test": false
    }))
    .expect("valid Target")
}

fn lib_target(src_path: &str) -> Target {
    from_value(serde_json::json!({
        "name": "my_crate",
        "kind": ["lib"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": true,
        "doctest": true,
        "test": true
    }))
    .expect("valid Target")
}

fn test_target(src_path: &str) -> Target {
    from_value(serde_json::json!({
        "name": "test_integration",
        "kind": ["test"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": false,
        "doctest": false,
        "test": true
    }))
    .expect("valid Target")
}

#[test]
fn collect_from_targets_bench_is_skipped() {
    // Arrange
    let target = bench_target("/project/benches/bench.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[target]);

    // Assert
    let roots = collector.into_roots();
    assert!(roots.is_empty());
}

#[test]
fn collect_from_targets_bin_adds_parent() {
    // Arrange
    let target = bin_target("/project/src/main.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[target]);

    // Assert
    let roots = collector.into_roots();
    assert_eq!(roots, vec![PathBuf::from("/project/src")]);
}

#[test]
fn collect_from_targets_custom_build_is_skipped() {
    // Arrange
    let target = custom_build_target("/project/build.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[target]);

    // Assert
    let roots = collector.into_roots();
    assert!(roots.is_empty());
}

#[test]
fn collect_from_targets_distinct_roots() {
    // Arrange
    let primary = lib_target("/project/src/lib.rs");
    let secondary = lib_target("/project/other/mod.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[primary, secondary]);

    // Assert
    let mut roots = collector.into_roots();
    roots.sort();
    assert_eq!(
        roots,
        vec![
            PathBuf::from("/project/other"),
            PathBuf::from("/project/src"),
        ]
    );
}

#[test]
fn collect_from_targets_example_is_skipped() {
    // Arrange
    let target = example_target("/project/examples/demo.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[target]);

    // Assert
    let roots = collector.into_roots();
    assert!(roots.is_empty());
}

#[test]
fn collect_from_targets_lib_adds_parent() {
    // Arrange
    let target = lib_target("/project/src/lib.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[target]);

    // Assert
    let roots = collector.into_roots();
    assert_eq!(roots, vec![PathBuf::from("/project/src")]);
}

#[test]
fn collect_from_targets_multiple_deduplicates() {
    // Arrange
    let lib = lib_target("/project/src/lib.rs");
    let bin = bin_target("/project/src/main.rs");
    let helper = lib_target("/project/src/helper.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[lib, bin, helper]);

    // Assert
    let roots = collector.into_roots();
    assert_eq!(roots, vec![PathBuf::from("/project/src")]);
}

#[test]
fn collect_from_targets_test_is_skipped() {
    // Arrange
    let target = test_target("/project/tests/integration.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[target]);

    // Assert
    let roots = collector.into_roots();
    assert!(roots.is_empty());
}

#[test]
fn ensure_fallback_empty_adds_src() {
    // Arrange
    let manifest_dir = PathBuf::from("/project");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.ensure_fallback(&manifest_dir);

    // Assert
    let roots = collector.into_roots();
    assert_eq!(roots, vec![PathBuf::from("/project/src")]);
}

#[test]
fn ensure_fallback_non_empty_does_not_add() {
    // Arrange
    let target = lib_target("/project/src/lib.rs");
    let manifest_dir = PathBuf::from("/project");
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[target]);

    // Act
    collector.ensure_fallback(&manifest_dir);

    // Assert
    let roots = collector.into_roots();
    assert_eq!(roots, vec![PathBuf::from("/project/src")]);
}

// A root nested inside another is redundant: SourceWalker recurses, so every
// file under it is already reached through the outer root. Left in, each of
// those files is walked, read and parsed twice, and reported twice.
#[test]
fn into_roots_drops_a_root_nested_inside_another() {
    // Arrange
    let library = lib_target("/p/src/lib.rs");
    let nested_binary = bin_target("/p/src/bin/board_ctl.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[library, nested_binary]);

    // Assert
    assert_eq!(collector.into_roots(), vec![PathBuf::from("/p/src")]);
}

#[test]
fn into_roots_drops_a_root_nested_several_levels_deep() {
    // Arrange
    let library = lib_target("/p/src/lib.rs");
    let deep_binary = bin_target("/p/src/tools/cli/entry.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[library, deep_binary]);

    // Assert
    assert_eq!(collector.into_roots(), vec![PathBuf::from("/p/src")]);
}

#[test]
fn into_roots_keeps_a_root_that_only_shares_a_name_prefix() {
    // Arrange
    let library = lib_target("/p/src/lib.rs");
    let lookalike = bin_target("/p/src_generated/entry.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[library, lookalike]);

    // Assert
    assert_eq!(
        collector.into_roots(),
        vec![PathBuf::from("/p/src"), PathBuf::from("/p/src_generated")]
    );
}

#[test]
fn into_roots_keeps_sibling_roots_that_do_not_nest() {
    // Arrange
    let library = lib_target("/p/src/lib.rs");
    let outside_binary = bin_target("/p/tools/probe.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[library, outside_binary]);

    // Assert
    assert_eq!(
        collector.into_roots(),
        vec![PathBuf::from("/p/src"), PathBuf::from("/p/tools")]
    );
}

#[test]
fn into_roots_returns_sorted_roots() {
    // Arrange
    let primary = lib_target("/z/src/lib.rs");
    let secondary = lib_target("/a/src/lib.rs");

    // Act
    let mut collector = TargetRootCollector::new();
    collector.collect_from_targets(&[primary, secondary]);

    // Assert
    let roots = collector.into_roots();
    let mut sorted = roots.clone();
    sorted.sort();
    assert_eq!(roots, sorted);
}

#[test]
fn is_production_target_lib_returns_true() {
    // Arrange & Act
    let result = TargetRootCollector::is_production_target(&lib_target("/p/src/lib.rs"));

    // Assert
    assert!(result);
}

#[test]
fn is_production_target_test_returns_false() {
    // Arrange & Act
    let result = TargetRootCollector::is_production_target(&test_target("/p/tests/t.rs"));

    // Assert
    assert!(!result);
}

#[test]
fn new_is_empty() {
    // Arrange
    let collector = TargetRootCollector::new();

    // Act
    let roots = collector.into_roots();

    // Assert
    assert!(roots.is_empty());
}
