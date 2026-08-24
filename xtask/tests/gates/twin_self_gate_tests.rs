// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_runner_tests::FakeCommandRunner;
use xtask::gates::gate::Gate;
use xtask::gates::twin_self_gate::TwinSelfGate;

fn gate(runner: &FakeCommandRunner) -> TwinSelfGate<'_> {
    TwinSelfGate::new(
        runner,
        String::from("cargo-twin4rust"),
        String::from("Cargo.toml"),
        vec![String::from("cargo-twin4rust")],
    )
}

// Everything before the `--` is cargo's; everything after it is the tool's.
fn split_at_separator(call: &[String]) -> (Vec<String>, Vec<String>) {
    let separator = call
        .iter()
        .position(|argument| argument == "--")
        .expect("the call separates cargo's arguments from the tool's");
    (call[..separator].to_vec(), call[separator + 1..].to_vec())
}

#[test]
fn label_names_the_self_analysis_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "twin4rust self-analysis");
}

#[test]
fn run_builds_the_tool_from_this_checkout_rather_than_an_install() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    let call = &runner.calls()[0];
    assert_eq!(call[0], "run");
    assert!(call.contains(&String::from("cargo-twin4rust")));
}

// Two flags that look alike sit on either side of the `--`. cargo gets `--bin`
// to choose what to build; the tool gets `--package` to choose what to analyse.
// Putting either on the wrong side is the mistake this pins: `--package` before
// the separator would have cargo pick a target, and it would never reach the
// tool that needs it.
#[test]
fn run_gives_cargo_the_binary_and_the_tool_the_package() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    let (for_cargo, for_tool) = split_at_separator(&runner.calls()[0]);
    assert!(for_cargo.contains(&String::from("--bin")));
    assert!(!for_cargo.contains(&String::from("--package")));
    assert!(for_tool.contains(&String::from("--package")));
    assert!(!for_tool.contains(&String::from("--bin")));
}

// The workspace root is a virtual manifest: it names no single package, so the
// tool refuses to guess and exits asking for one. Without this argument the
// gate fails on every run.
#[test]
fn run_names_the_package_the_virtual_manifest_cannot_imply() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    let (_, for_tool) = split_at_separator(&runner.calls()[0]);
    assert!(for_tool.contains(&String::from("cargo-twin4rust")));
}

#[test]
fn run_passes_the_manifest_path_to_the_tool() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    let call = &runner.calls()[0];
    assert!(call.contains(&String::from("--manifest-path")));
    assert!(call.contains(&String::from("Cargo.toml")));
}

#[test]
fn run_with_a_non_zero_exit_code_reports_a_missing_mirrored_test() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(String::from("source files without a mirrored test"))
    );
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_ok());
}

// None means the process was signalled rather than exiting. That is a failure,
// not a pass, and the arm that treats it as one would be easy to write.
#[test]
fn run_with_no_exit_code_is_a_failure_rather_than_a_pass() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(None);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err());
}
