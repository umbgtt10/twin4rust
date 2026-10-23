// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use twin4rust::analysis_report::AnalysisReport;
use twin4rust::config::Config;
use twin4rust::missing_test_gap::MissingTestGap;
use twin4rust::package_context::PackageContext;

#[test]
fn analysis_report_clone_produces_independent_copy() {
    // Arrange
    let gap = MissingTestGap::new(
        "p".to_string(),
        "f.rs".to_string(),
        "f_tests.rs".to_string(),
    );
    let report = AnalysisReport::new("p".to_string(), vec![gap]);

    // Act
    let cloned = report.clone();

    // Assert
    assert_eq!(cloned.package_name(), report.package_name());
    assert_eq!(cloned.missing(), report.missing());
}

#[test]
fn analysis_report_is_empty_returns_false_when_missing_exists() {
    // Arrange
    let gap = MissingTestGap::new(
        "p".to_string(),
        "f.rs".to_string(),
        "f_tests.rs".to_string(),
    );

    // Act
    let report = AnalysisReport::new("p".to_string(), vec![gap]);

    // Assert
    assert!(!report.is_empty());
}

#[test]
fn analysis_report_is_empty_returns_true_when_no_missing() {
    // Arrange & Act
    let report = AnalysisReport::new("any".to_string(), vec![]);

    // Assert
    assert!(report.is_empty());
}

#[test]
fn analysis_report_missing_count_returns_correct_number() {
    // Arrange
    let gaps = vec![
        MissingTestGap::new(
            "p".to_string(),
            "a.rs".to_string(),
            "a_tests.rs".to_string(),
        ),
        MissingTestGap::new(
            "p".to_string(),
            "b.rs".to_string(),
            "b_tests.rs".to_string(),
        ),
    ];

    // Act
    let report = AnalysisReport::new("p".to_string(), gaps);

    // Assert
    assert_eq!(report.missing_count(), 2);
}

#[test]
fn analysis_report_new_creates_with_empty_missing() {
    // Arrange & Act
    let report = AnalysisReport::new("pkg".to_string(), vec![]);

    // Assert
    assert_eq!(report.package_name(), "pkg");
    assert!(report.is_empty());
    assert_eq!(report.missing_count(), 0);
}

#[test]
fn analysis_report_new_with_gaps_is_not_empty() {
    // Arrange
    let gap = MissingTestGap::new(
        "pkg".to_string(),
        "src/lib.rs".to_string(),
        "tests/lib_tests.rs".to_string(),
    );

    // Act
    let report = AnalysisReport::new("pkg".to_string(), vec![gap]);

    // Assert
    assert!(!report.is_empty());
    assert_eq!(report.missing_count(), 1);
}

#[test]
fn analysis_report_partial_eq_compares_fields() {
    // Arrange
    let a = AnalysisReport::new("pkg".to_string(), vec![]);
    let b = AnalysisReport::new("pkg".to_string(), vec![]);
    let c = AnalysisReport::new("other".to_string(), vec![]);

    // Act & Assert
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn config_clone_produces_independent_copy() {
    // Arrange
    let config = Config::new(Some(PathBuf::from("Cargo.toml")), vec!["pkg".to_string()]);

    // Act
    let cloned = config.clone();

    // Assert
    assert_eq!(cloned.manifest_path(), config.manifest_path());
    assert_eq!(cloned.packages(), config.packages());
}

#[test]
fn config_default_manifest_path_is_none() {
    // Arrange & Act
    let config = Config::new(None, vec![]);

    // Assert
    assert!(config.manifest_path().is_none());
}

#[test]
fn config_empty_packages_is_empty_vec() {
    // Arrange & Act
    let config = Config::new(None, vec![]);

    // Assert
    assert!(config.packages().is_empty());
}

#[test]
fn config_holds_provided_values() {
    // Arrange & Act
    let config = Config::new(Some(PathBuf::from("Cargo.toml")), vec!["foo".to_string()]);

    // Assert
    assert_eq!(
        config.manifest_path().unwrap().to_string_lossy(),
        "Cargo.toml"
    );
    assert_eq!(config.packages(), vec!["foo"]);
}

#[test]
fn missing_test_gap_clone_produces_independent_copy() {
    // Arrange
    let gap = MissingTestGap::new(
        "pkg".to_string(),
        "src/main.rs".to_string(),
        "tests/main_tests.rs".to_string(),
    );

    // Act
    let cloned = gap.clone();

    // Assert
    assert_eq!(cloned, gap);
}

#[test]
fn missing_test_gap_holds_provided_values() {
    // Arrange & Act
    let gap = MissingTestGap::new(
        "pkg".to_string(),
        "src/lib.rs".to_string(),
        "tests/lib_tests.rs".to_string(),
    );

    // Assert
    assert_eq!(gap.package_name(), "pkg");
    assert_eq!(gap.relative_source_file(), "src/lib.rs");
    assert_eq!(gap.expected_test_file(), "tests/lib_tests.rs");
}

#[test]
fn missing_test_gap_partial_eq_compares_all_fields() {
    // Arrange
    let a = MissingTestGap::new(
        "pkg".to_string(),
        "src/a.rs".to_string(),
        "tests/a_tests.rs".to_string(),
    );
    let b = MissingTestGap::new(
        "pkg".to_string(),
        "src/a.rs".to_string(),
        "tests/a_tests.rs".to_string(),
    );
    let c = MissingTestGap::new(
        "other".to_string(),
        a.relative_source_file().to_string(),
        a.expected_test_file().to_string(),
    );

    // Act & Assert
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn package_context_clone_produces_independent_copy() {
    // Arrange
    let ctx = PackageContext::new(
        "pkg".to_string(),
        PathBuf::from("/a"),
        vec![PathBuf::from("/a/src")],
    );

    // Act
    let cloned = ctx.clone();

    // Assert
    assert_eq!(cloned.name(), ctx.name());
    assert_eq!(cloned.manifest_dir(), ctx.manifest_dir());
    assert_eq!(cloned.source_roots(), ctx.source_roots());
}

#[test]
fn package_context_holds_provided_values() {
    // Arrange & Act
    let ctx = PackageContext::new(
        "my-crate".to_string(),
        PathBuf::from("/home/project"),
        vec![PathBuf::from("/home/project/src")],
    );

    // Assert
    assert_eq!(ctx.name(), "my-crate");
    assert_eq!(ctx.manifest_dir(), PathBuf::from("/home/project"));
    assert_eq!(ctx.source_roots(), vec![PathBuf::from("/home/project/src")]);
}

#[test]
fn package_context_multiple_source_roots() {
    // Arrange & Act
    let ctx = PackageContext::new(
        "workspace".to_string(),
        PathBuf::from("/ws"),
        vec![
            PathBuf::from("/ws/crate_a/src"),
            PathBuf::from("/ws/crate_b/src"),
        ],
    );

    // Assert
    assert_eq!(ctx.source_roots().len(), 2);
}
