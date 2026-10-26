// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::InMemorySourceTree;
use crate::support::in_memory_workspace_reader::InMemoryWorkspaceReader;
use crate::support::in_memory_workspace_reader::library_package;
use crate::support::summary_report_renderer::SummaryReportRenderer;
use twin4rust::config::Config;
use twin4rust::runner::Runner;
use twin4rust::text_report_renderer::TextReportRenderer;
use twin4rust::workspace::workspace_description::WorkspaceDescription;

const BEHAVIOUR: &str = "pub fn start() -> u32 {\n    1\n}\n";

fn config() -> Config {
    Config {
        packages: vec![String::from("alpha"), String::from("beta")],
    }
}

fn two_package_workspace() -> InMemoryWorkspaceReader {
    InMemoryWorkspaceReader::new(WorkspaceDescription {
        packages: vec![
            library_package("alpha", "/w/alpha"),
            library_package("beta", "/w/beta"),
        ],
        root_package: None,
    })
}

#[test]
fn run_hands_every_package_report_to_the_renderer_in_the_order_asked() {
    // Arrange
    let reader = two_package_workspace();
    let tree = InMemorySourceTree::new()
        .with_file("/w/alpha/src/engine.rs", BEHAVIOUR)
        .with_file("/w/beta/src/engine.rs", BEHAVIOUR)
        .with_file("/w/beta/src/wheel.rs", BEHAVIOUR);

    // Act
    let outcome = Runner::new(&reader, &tree, &SummaryReportRenderer)
        .run(&config())
        .expect("run");

    // Assert
    assert_eq!(outcome.output, "alpha=1\nbeta=2\n");
}

#[test]
fn run_over_packages_missing_a_mirror_reports_the_gaps() {
    // Arrange
    let reader = two_package_workspace();
    let tree = InMemorySourceTree::new()
        .with_file("/w/alpha/src/engine.rs", BEHAVIOUR)
        .with_file("/w/alpha/tests/engine_tests.rs", "")
        .with_file("/w/beta/src/engine.rs", BEHAVIOUR);

    // Act
    let outcome = Runner::new(&reader, &tree, &TextReportRenderer::new())
        .run(&config())
        .expect("run");

    // Assert
    assert!(outcome.has_gaps);
    assert_eq!(
        outcome.output,
        "twin4rust report\n\nFiles without a matching mirrored test file:\n- beta: src/engine.rs -> tests/engine_tests.rs\n\nsummary: packages_with_gaps=1 missing_files=1\n"
    );
}

#[test]
fn run_over_packages_whose_files_all_have_mirrors_reports_no_gaps() {
    // Arrange
    let reader = two_package_workspace();
    let tree = InMemorySourceTree::new()
        .with_file("/w/alpha/src/engine.rs", BEHAVIOUR)
        .with_file("/w/alpha/tests/engine_tests.rs", "");

    // Act
    let outcome = Runner::new(&reader, &tree, &TextReportRenderer::new())
        .run(&config())
        .expect("run");

    // Assert
    assert!(!outcome.has_gaps);
    assert_eq!(
        outcome.output,
        "twin4rust report\n\nAll mirrored test-gap expectations are satisfied.\n"
    );
}

#[test]
fn run_with_a_workspace_that_cannot_be_read_returns_the_error() {
    // Arrange
    let reader = InMemoryWorkspaceReader::failing();
    let tree = InMemorySourceTree::new();

    // Act
    let result = Runner::new(&reader, &tree, &TextReportRenderer::new()).run(&config());

    // Assert
    assert_eq!(
        result.err().map(|failure| failure.to_string()),
        Some(String::from("failed to read Cargo metadata"))
    );
}

#[test]
fn runner_exists_and_is_send() {
    // Arrange & Act
    fn assert_send<T: Send>() {}
    assert_send::<Runner>();

    // Assert
    // Compile-time check: Runner is constructible and Send
}
