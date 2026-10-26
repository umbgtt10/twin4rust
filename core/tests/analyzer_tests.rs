// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::InMemorySourceTree;
use twin4rust::analyzer::Analyzer;
use twin4rust::package_context::PackageContext;
use twin4rust::workspace::source_path::SourcePath;

fn package_context(root: &str, name: &str) -> PackageContext {
    let manifest_dir = SourcePath::new(String::from(root));
    PackageContext {
        name: name.to_string(),
        source_roots: vec![manifest_dir.join("src")],
        manifest_dir,
    }
}

// `src/bin/*.rs` is Cargo's location for extra binaries, so each file directly
// under it is an entry point in exactly the sense `src/main.rs` is. Excluding
// only the three literal paths `src/lib.rs`, `src/main.rs` and `build.rs`
// leaves these demanding a mirrored test for a `fn main`.
#[test]
fn a_binary_entry_point_under_src_bin_is_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/bin/board_ctl.rs",
        r#"
fn main() {
    if std::env::args().count() > 1 {
        println!("with arguments");
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "a binary entry point under src/bin should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn a_humble_adapter_forwarding_to_an_untestable_boundary_is_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/board_halter.rs",
        r#"
use crate::board_eraser::BoardEraser;
use crate::board_id::BoardId;

pub struct BoardHalter {
    board_id: BoardId,
    probe_serial: String,
}

impl BoardHalter {
    pub fn new(board_id: BoardId, probe_serial: &str) -> Self {
        Self { board_id, probe_serial: probe_serial.to_string() }
    }

    pub fn halt(self) {
        BoardEraser::new(self.board_id, &self.probe_serial).erase();
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "a constructor plus a forwarding method has nothing to mirror, got: {:?}",
        report.missing()
    );
}

// The boundary: only files directly under src/bin are entry points. A module
// belonging to a src/bin/<name>/main.rs binary carries ordinary behaviour and
// must stay in scope, or the fix would silently exempt a whole subtree.
#[test]
fn a_module_belonging_to_a_src_bin_binary_is_still_reported() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/bin/tool/main.rs",
        r#"
fn main() {
    println!("tool");
}
"#,
    );
    let tree = tree.with_file(
        "/p/src/bin/tool/parser.rs",
        r#"
pub fn parse(input: &str) -> usize {
    if input.is_empty() { 0 } else { input.len() }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert_eq!(
        report.missing().len(),
        1,
        "only the parser module should be reported, got: {:?}",
        report.missing()
    );
    assert!(
        report.missing()[0]
            .relative_source_file
            .replace(std::path::MAIN_SEPARATOR, "/")
            .ends_with("src/bin/tool/parser.rs"),
        "expected the parser module, got: {:?}",
        report.missing()
    );
}

#[test]
fn a_single_type_file_whose_method_returns_a_value_is_still_reported() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/rtt_channel_config.rs",
        r#"
pub struct RttChannelConfig {
    buffer_size: RttBufferSize,
    level: FirmwareLogLevel,
}

impl RttChannelConfig {
    pub fn to_label(&self) -> String {
        format!("{}@{}", self.buffer_size.label(), self.level.label())
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert_eq!(
        report.missing().len(),
        1,
        "a method producing a value must stay in scope, got: {:?}",
        report.missing()
    );
}

#[test]
fn an_adapter_that_grows_a_branch_is_reported_again() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/board_halter.rs",
        r#"
pub struct BoardHalter {
    board_id: BoardId,
    probe_serial: String,
}

impl BoardHalter {
    pub fn halt(self) {
        if self.probe_serial.is_empty() {
            return;
        }
        BoardEraser::new(self.board_id, &self.probe_serial).erase();
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert_eq!(
        report.missing().len(),
        1,
        "an adapter carrying a decision must be reported, got: {:?}",
        report.missing()
    );
}

#[test]
fn analyze_package_over_a_source_root_that_does_not_exist_returns_an_empty_report() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file("/elsewhere/src/lib.rs", "pub fn f() {}\n");

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert_eq!(report.package_name(), "demo-node");
    assert!(report.is_empty());
}

#[test]
fn analyze_package_with_a_file_that_does_not_parse_returns_an_error() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file("/p/src/broken.rs", "pub fn {");

    // Act
    let result = Analyzer::new(&tree).analyze_package(&package_context("/p", "demo-node"));

    // Assert
    assert_eq!(
        result.err().map(|failure| failure.to_string()),
        Some(String::from("failed to parse Rust source"))
    );
}

#[test]
fn analyze_package_with_a_mirrored_test_file_reports_no_gap() {
    // Arrange
    let tree = InMemorySourceTree::new()
        .with_file(
            "/p/src/implementations/raft/client.rs",
            "pub fn start_client() {}\n",
        )
        .with_file("/p/tests/implementations/raft/client_tests.rs", "");

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "raft-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "a source file whose mirror exists has no gap, got: {:?}",
        report.missing()
    );
}

#[test]
fn analyze_package_with_an_unreadable_file_names_it_in_the_error() {
    // Arrange
    let tree = InMemorySourceTree::new().with_unreadable_file("/p/src/locked.rs");

    // Act
    let result = Analyzer::new(&tree).analyze_package(&package_context("/p", "demo-node"));

    // Assert
    assert_eq!(
        result.err().map(|failure| failure.to_string()),
        Some(String::from("failed to read source file /p/src/locked.rs"))
    );
}

#[test]
fn analyze_packages_over_no_packages_returns_no_reports() {
    // Arrange
    let tree = InMemorySourceTree::new();
    let analyzer = Analyzer::new(&tree);

    // Act
    let reports = analyzer.analyze_packages(&[]).expect("analyze");

    // Assert
    assert!(reports.is_empty());
}

#[test]
fn analyze_packages_over_two_packages_returns_a_report_for_each() {
    // Arrange
    let tree = InMemorySourceTree::new()
        .with_file("/first/src/lib.rs", "pub fn f() {}\n")
        .with_file("/second/src/lib.rs", "pub fn f() {}\n");
    let packages = vec![
        package_context("/first", "first"),
        package_context("/second", "second"),
    ];
    let analyzer = Analyzer::new(&tree);

    // Act
    let reports = analyzer.analyze_packages(&packages).expect("analyze");

    // Assert
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].package_name(), "first");
    assert_eq!(reports[1].package_name(), "second");
}

#[test]
fn behavioral_validation_crate_source_is_still_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/in_memory_transport.rs",
        r#"
pub struct InMemoryTransport;

impl InMemoryTransport {
    pub fn receive(&self) -> bool {
        true
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "raft-validation"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "validation crate source should still be ignored even when behavioral, got: {:?}",
        report.missing()
    );
}

#[test]
fn enum_with_non_struct_new_body_is_not_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/status.rs",
        r#"
pub enum Status {
    Active,
    Inactive,
}

impl Status {
    pub fn new() -> Self {
        Self::Active
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing()
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/status.rs"),
        "enum with non-struct body in new() should not be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn generic_struct_with_new_is_ignored_as_trivial_constructor() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/wrapper.rs",
        r#"
pub struct Wrapper<T> {
    pub inner: T,
}

impl<T> Wrapper<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "struct with generic trivial new constructor should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn import_only_mod_file_is_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/something/mod.rs",
        r#"
pub mod child;
use crate::foo::Bar;
pub use crate::baz::Qux;
"#,
    );
    let tree = tree.with_file(
        "/p/src/something/child.rs",
        r#"
pub fn do_work() {}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    let missing_files = report
        .missing()
        .iter()
        .map(|gap| gap.relative_source_file.as_str())
        .collect::<Vec<_>>();

    assert!(
        !missing_files.contains(&"src/something/mod.rs"),
        "import-only mod.rs should be ignored, got: {:?}",
        missing_files
    );
    assert!(
        missing_files.contains(&"src/something/child.rs"),
        "non-ignored child source should still be reported, got: {:?}",
        missing_files
    );
}

#[test]
fn lib_main_and_build_files_are_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/lib.rs",
        r#"
pub fn lib_entry() {}
"#,
    );
    let tree = tree.with_file(
        "/p/src/main.rs",
        r#"
pub fn main_entry() {}
"#,
    );
    let tree = tree.with_file(
        "/p/build.rs",
        r#"
fn main() {}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "lib.rs, main.rs, and build.rs should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn mirrored_test_path_is_detected_for_nested_non_ignored_source_file() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/implementations/raft/client.rs",
        r#"
pub fn start_client() {}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "raft-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.missing().iter().any(|gap| {
            gap.relative_source_file == "src/implementations/raft/client.rs"
                && gap.expected_test_file == "tests/implementations/raft/client_tests.rs"
        }),
        "expected nested mirrored path to be reported, got: {:?}",
        report.missing()
    );
}

#[test]
fn mod_file_is_ignored_under_current_gate_policy() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/feature/mod.rs",
        r#"
pub mod child;

pub fn configure() {}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        !report
            .missing()
            .iter()
            .any(|gap| gap.relative_source_file == "src/feature/mod.rs"),
        "mod.rs files should be ignored under the current gate policy, got: {:?}",
        report.missing()
    );
}

#[test]
fn multiple_definition_only_items_are_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/types.rs",
        r#"
pub trait QueryAdapter {
    type Output;
}

pub struct QueryEnvelope {
    pub value: u64,
}

pub enum QueryKind {
    Height,
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "multiple definition-only items should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn nontrivial_new_constructor_is_not_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/response.rs",
        r#"
pub struct Response {
    pub value: u64,
}

impl Response {
    pub fn new(value: u64) -> Self {
        if value == 0 {
            Self { value: 1 }
        } else {
            Self { value }
        }
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing()
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/response.rs"),
        "nontrivial new constructor should not be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn pure_single_enum_definition_is_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/state/storage/storage_query_result.rs",
        r#"
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum StorageQueryResult {
    Height(u64),
    Accounts(BTreeMap<u64, u64>),
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "single enum definition file should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn pure_single_struct_definition_is_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/node_role.rs",
        r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRole {
    pub leader: bool,
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "raft-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "single struct definition file should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn several_binary_entry_points_under_src_bin_are_all_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/bin/first.rs",
        r#"
fn main() {
    for argument in std::env::args() {
        println!("{argument}");
    }
}
"#,
    );
    let tree = tree.with_file(
        "/p/src/bin/second.rs",
        r#"
fn main() {
    if cfg!(debug_assertions) {
        println!("debug");
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "every binary entry point under src/bin should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn single_enum_with_nontrivial_impl_is_not_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/message_source.rs",
        r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSource {
    Peer(u64),
    Client(u64),
}

impl MessageSource {
    pub fn is_peer(&self) -> bool {
        match self {
            Self::Peer(_) => true,
            Self::Client(_) => false,
        }
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing()
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/message_source.rs"),
        "enum with nontrivial impl should not be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn single_struct_with_constructor_not_named_new_is_not_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/builder.rs",
        r#"
pub struct Builder {
    pub value: u64,
}

impl Builder {
    pub fn build(value: u64) -> Self {
        Self { value }
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing()
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/builder.rs"),
        "struct with constructor not named new should not be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn single_struct_with_empty_impl_is_not_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/marker.rs",
        r#"
pub struct Marker;

impl Marker {
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        !report.is_empty(),
        "struct with empty impl should not be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn single_struct_with_multiple_impl_methods_is_not_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/snapshot.rs",
        r#"
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub height: u64,
}

impl Snapshot {
    pub fn new(height: u64) -> Self {
        Self { height }
    }

    pub fn is_empty(&self) -> bool {
        self.height == 0
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "raft-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing()
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/snapshot.rs"),
        "struct with multiple impl methods should not be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn single_struct_with_trait_impl_is_not_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/convertible.rs",
        r#"
pub struct Convertible;

impl From<u64> for Convertible {
    fn from(value: u64) -> Self {
        Convertible
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing()
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/convertible.rs"),
        "struct with trait impl should not be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn single_struct_with_trivial_new_constructor_is_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/implementations/recovery/peer_recovery_status.rs",
        r#"
use crate::common_types::types::{Hash, Height};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerRecoveryStatus {
    pub height: Height,
    pub last_hash: Hash,
}

impl PeerRecoveryStatus {
    pub fn new(height: Height, last_hash: Hash) -> Self {
        Self { height, last_hash }
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "single struct with trivial new constructor should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn source_file_with_no_impl_block_is_definition_only() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/point.rs",
        r#"
pub struct Point {
    pub x: f64,
    pub y: f64,
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "struct with no impl should be ignored as definition-only, got: {:?}",
        report.missing()
    );
}

#[test]
fn trait_and_type_definition_only_file_is_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/storage_adapter.rs",
        r#"
pub trait StorageAdapter {
    type Query;
}

pub type StorageKey = u64;
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "trait/type definition-only file should be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn trait_impl_for_single_type_is_not_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/common_types/displayable.rs",
        r#"
pub trait Printable {
    fn render(&self) -> &'static str;
}

pub struct Displayable;

impl Printable for Displayable {
    fn render(&self) -> &'static str {
        "displayable"
    }
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing()
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/displayable.rs"),
        "trait impl for a single type should not be ignored, got: {:?}",
        report.missing()
    );
}

#[test]
fn validation_crate_src_files_are_ignored() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/p/src/protocol_cluster.rs",
        r#"
use std::collections::BTreeMap;

pub fn helper() -> usize {
    42
}
"#,
    );

    // Act
    let report = Analyzer::new(&tree)
        .analyze_package(&package_context("/p", "demo-validation"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "validation crate src files should be ignored entirely, got: {:?}",
        report.missing()
    );
}
