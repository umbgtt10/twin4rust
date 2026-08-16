// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use twin4rust::analysis_report::AnalysisReport;
use twin4rust::missing_test_gap::MissingTestGap;
use twin4rust::report_printer::ReportPrinter;

#[test]
fn print_empty_reports_does_not_panic() {
    // Arrange
    let reports: Vec<AnalysisReport> = vec![];
    let printer = ReportPrinter::new();

    // Act & Assert
    let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        printer.print(&reports);
    }));
    assert!(output.is_ok());
}

#[test]
fn print_reports_with_no_gaps_does_not_panic() {
    // Arrange
    let reports = vec![AnalysisReport::new("pkg".to_string(), vec![])];
    let printer = ReportPrinter::new();

    // Act & Assert
    let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        printer.print(&reports);
    }));
    assert!(output.is_ok());
}

#[test]
fn print_reports_with_missing_gaps_does_not_panic() {
    // Arrange
    let missing = vec![MissingTestGap {
        package_name: "pkg".to_string(),
        relative_source_file: "src/lib.rs".to_string(),
        expected_test_file: "tests/lib_tests.rs".to_string(),
    }];
    let reports = vec![AnalysisReport::new("pkg".to_string(), missing)];
    let printer = ReportPrinter::new();

    // Act & Assert
    let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        printer.print(&reports);
    }));
    assert!(output.is_ok());
}

#[test]
fn print_reports_with_multiple_packages_and_gaps_does_not_panic() {
    // Arrange
    let missing_a = vec![
        MissingTestGap {
            package_name: "pkg-a".to_string(),
            relative_source_file: "src/foo.rs".to_string(),
            expected_test_file: "tests/foo_tests.rs".to_string(),
        },
        MissingTestGap {
            package_name: "pkg-a".to_string(),
            relative_source_file: "src/bar.rs".to_string(),
            expected_test_file: "tests/bar_tests.rs".to_string(),
        },
    ];
    let missing_b = vec![MissingTestGap {
        package_name: "pkg-b".to_string(),
        relative_source_file: "src/baz.rs".to_string(),
        expected_test_file: "tests/baz_tests.rs".to_string(),
    }];
    let reports = vec![
        AnalysisReport::new("pkg-a".to_string(), missing_a),
        AnalysisReport::new("pkg-b".to_string(), missing_b),
    ];
    let printer = ReportPrinter::new();

    // Act & Assert
    let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        printer.print(&reports);
    }));
    assert!(output.is_ok());
}
