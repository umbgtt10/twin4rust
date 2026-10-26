// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use twin4rust::analysis_report::AnalysisReport;
use twin4rust::missing_test_gap::MissingTestGap;
use twin4rust::text_report_renderer::TextReportRenderer;
use twin4rust::traits::report_renderer::ReportRenderer;

fn gap(package_name: &str, stem: &str) -> MissingTestGap {
    MissingTestGap {
        package_name: package_name.to_string(),
        relative_source_file: format!("src/{stem}.rs"),
        expected_test_file: format!("tests/{stem}_tests.rs"),
    }
}

#[test]
fn render_empty_reports_says_every_expectation_is_satisfied() {
    // Arrange
    let reports: Vec<AnalysisReport> = vec![];

    // Act
    let output = TextReportRenderer::new().render(&reports);

    // Assert
    assert_eq!(
        output,
        "twin4rust report\n\nAll mirrored test-gap expectations are satisfied.\n"
    );
}

#[test]
fn render_reports_with_missing_gaps_lists_each_and_the_summary() {
    // Arrange
    let reports = vec![AnalysisReport::new(
        "pkg".to_string(),
        vec![gap("pkg", "lib")],
    )];

    // Act
    let output = TextReportRenderer::new().render(&reports);

    // Assert
    assert_eq!(
        output,
        "twin4rust report\n\nFiles without a matching mirrored test file:\n- pkg: src/lib.rs -> tests/lib_tests.rs\n\nsummary: packages_with_gaps=1 missing_files=1\n"
    );
}

#[test]
fn render_reports_with_multiple_packages_and_gaps_sorts_them_and_counts_both() {
    // Arrange
    let reports = vec![
        AnalysisReport::new(
            "pkg-a".to_string(),
            vec![gap("pkg-a", "foo"), gap("pkg-a", "bar")],
        ),
        AnalysisReport::new("pkg-b".to_string(), vec![gap("pkg-b", "baz")]),
    ];

    // Act
    let output = TextReportRenderer::new().render(&reports);

    // Assert
    assert_eq!(
        output,
        "twin4rust report\n\nFiles without a matching mirrored test file:\n- pkg-a: src/bar.rs -> tests/bar_tests.rs\n- pkg-a: src/foo.rs -> tests/foo_tests.rs\n- pkg-b: src/baz.rs -> tests/baz_tests.rs\n\nsummary: packages_with_gaps=2 missing_files=3\n"
    );
}

#[test]
fn render_reports_with_no_gaps_says_every_expectation_is_satisfied() {
    // Arrange
    let reports = vec![AnalysisReport::new("pkg".to_string(), vec![])];

    // Act
    let output = TextReportRenderer::new().render(&reports);

    // Assert
    assert_eq!(
        output,
        "twin4rust report\n\nAll mirrored test-gap expectations are satisfied.\n"
    );
}
