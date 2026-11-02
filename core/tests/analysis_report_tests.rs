// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use twin4rust::analysis_report::AnalysisReport;
use twin4rust::missing_test_gap::MissingTestGap;
use twin4rust::package_context::PackageContext;
use twin4rust::unearned_mirror::UnearnedMirror;
use twin4rust::workspace::source_path::SourcePath;

#[test]
fn analysis_report_clone_produces_independent_copy() {
    // Arrange
    let gap = MissingTestGap {
        package_name: "p".to_string(),
        relative_source_file: "f.rs".to_string(),
        expected_test_file: "f_tests.rs".to_string(),
    };
    let report = AnalysisReport::new("p".to_string(), vec![gap], vec![]);

    // Act
    let cloned = report.clone();

    // Assert
    assert_eq!(cloned.package_name(), report.package_name());
    assert_eq!(cloned.missing(), report.missing());
}

#[test]
fn analysis_report_is_empty_returns_false_when_an_unearned_mirror_exists() {
    // Arrange
    let mirror = UnearnedMirror {
        package_name: "p".to_string(),
        relative_source_file: "src/window.rs".to_string(),
        mirror_test_file: "tests/window_tests.rs".to_string(),
    };

    // Act
    let report = AnalysisReport::new("p".to_string(), vec![], vec![mirror]);

    // Assert
    assert!(!report.is_empty());
}

#[test]
fn analysis_report_is_empty_returns_false_when_missing_exists() {
    // Arrange
    let gap = MissingTestGap {
        package_name: "p".to_string(),
        relative_source_file: "f.rs".to_string(),
        expected_test_file: "f_tests.rs".to_string(),
    };

    // Act
    let report = AnalysisReport::new("p".to_string(), vec![gap], vec![]);

    // Assert
    assert!(!report.is_empty());
}

#[test]
fn analysis_report_is_empty_returns_true_when_no_missing() {
    // Arrange & Act
    let report = AnalysisReport::new("any".to_string(), vec![], vec![]);

    // Assert
    assert!(report.is_empty());
}

#[test]
fn analysis_report_missing_count_returns_correct_number() {
    // Arrange
    let gaps = vec![
        MissingTestGap {
            package_name: "p".to_string(),
            relative_source_file: "a.rs".to_string(),
            expected_test_file: "a_tests.rs".to_string(),
        },
        MissingTestGap {
            package_name: "p".to_string(),
            relative_source_file: "b.rs".to_string(),
            expected_test_file: "b_tests.rs".to_string(),
        },
    ];

    // Act
    let report = AnalysisReport::new("p".to_string(), gaps, vec![]);

    // Assert
    assert_eq!(report.missing_count(), 2);
}

#[test]
fn analysis_report_new_creates_with_empty_missing() {
    // Arrange & Act
    let report = AnalysisReport::new("pkg".to_string(), vec![], vec![]);

    // Assert
    assert_eq!(report.package_name(), "pkg");
    assert!(report.is_empty());
    assert_eq!(report.missing_count(), 0);
}

#[test]
fn analysis_report_new_with_gaps_is_not_empty() {
    // Arrange
    let gap = MissingTestGap {
        package_name: "pkg".to_string(),
        relative_source_file: "src/lib.rs".to_string(),
        expected_test_file: "tests/lib_tests.rs".to_string(),
    };

    // Act
    let report = AnalysisReport::new("pkg".to_string(), vec![gap], vec![]);

    // Assert
    assert!(!report.is_empty());
    assert_eq!(report.missing_count(), 1);
}

#[test]
fn analysis_report_partial_eq_compares_fields() {
    // Arrange
    let a = AnalysisReport::new("pkg".to_string(), vec![], vec![]);
    let b = AnalysisReport::new("pkg".to_string(), vec![], vec![]);
    let c = AnalysisReport::new("other".to_string(), vec![], vec![]);

    // Act & Assert
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn analysis_report_unearned_count_returns_the_number_of_unearned_mirrors() {
    // Arrange
    let mirror = UnearnedMirror {
        package_name: "p".to_string(),
        relative_source_file: "src/window.rs".to_string(),
        mirror_test_file: "tests/window_tests.rs".to_string(),
    };
    let report = AnalysisReport::new("p".to_string(), vec![], vec![mirror.clone(), mirror]);

    // Act
    let count = report.unearned_count();

    // Assert
    assert_eq!(count, 2);
}

#[test]
fn analysis_report_unearned_returns_the_unearned_mirrors() {
    // Arrange
    let mirror = UnearnedMirror {
        package_name: "p".to_string(),
        relative_source_file: "src/window.rs".to_string(),
        mirror_test_file: "tests/window_tests.rs".to_string(),
    };
    let report = AnalysisReport::new("p".to_string(), vec![], vec![mirror.clone()]);

    // Act
    let unearned = report.unearned();

    // Assert
    assert_eq!(unearned, &[mirror]);
}

#[test]
fn package_context_clone_produces_independent_copy() {
    // Arrange
    let ctx = PackageContext {
        name: "pkg".to_string(),
        manifest_dir: SourcePath::new(String::from("/a")),
        source_roots: vec![SourcePath::new(String::from("/a/src"))],
    };

    // Act
    let cloned = ctx.clone();

    // Assert
    assert_eq!(cloned.name, ctx.name);
    assert_eq!(cloned.manifest_dir, ctx.manifest_dir);
    assert_eq!(cloned.source_roots, ctx.source_roots);
}
