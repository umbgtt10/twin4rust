// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::env::temp_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use twin4rust::analyzer::Analyzer;
use twin4rust::package_context::PackageContext;

fn analyzer() -> Analyzer {
    Analyzer::new()
}

fn package_context(root: &Path, name: &str) -> PackageContext {
    PackageContext {
        name: name.to_string(),
        manifest_dir: root.to_path_buf(),
        source_roots: vec![root.join("src")],
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();

    let dir = temp_dir().join(format!("twin4rust_{label}_{nanos}"));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn write_file(root: &Path, relative_path: &str, contents: &str) {
    let full_path = root.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(full_path, contents).expect("failed to write file");
}

// `src/bin/*.rs` is Cargo's location for extra binaries, so each file directly
// under it is an entry point in exactly the sense `src/main.rs` is. Excluding
// only the three literal paths `src/lib.rs`, `src/main.rs` and `build.rs`
// leaves these demanding a mirrored test for a `fn main`.
#[test]
fn a_binary_entry_point_under_src_bin_is_ignored() {
    // Arrange
    let root = unique_temp_dir("src_bin_entry_point");
    write_file(
        &root,
        "src/bin/board_ctl.rs",
        r#"
fn main() {
    if std::env::args().count() > 1 {
        println!("with arguments");
    }
}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "a binary entry point under src/bin should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn a_humble_adapter_forwarding_to_an_untestable_boundary_is_ignored() {
    // Arrange
    let root = unique_temp_dir("humble_adapter_ignored");
    write_file(
        &root,
        "src/board_halter.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "a constructor plus a forwarding method has nothing to mirror, got: {:?}",
        report.missing
    );
}

// The boundary: only files directly under src/bin are entry points. A module
// belonging to a src/bin/<name>/main.rs binary carries ordinary behaviour and
// must stay in scope, or the fix would silently exempt a whole subtree.
#[test]
fn a_module_belonging_to_a_src_bin_binary_is_still_reported() {
    // Arrange
    let root = unique_temp_dir("src_bin_module_reported");
    write_file(
        &root,
        "src/bin/tool/main.rs",
        r#"
fn main() {
    println!("tool");
}
"#,
    );
    write_file(
        &root,
        "src/bin/tool/parser.rs",
        r#"
pub fn parse(input: &str) -> usize {
    if input.is_empty() { 0 } else { input.len() }
}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert_eq!(
        report.missing.len(),
        1,
        "only the parser module should be reported, got: {:?}",
        report.missing
    );
    assert!(
        report.missing[0]
            .relative_source_file
            .replace(std::path::MAIN_SEPARATOR, "/")
            .ends_with("src/bin/tool/parser.rs"),
        "expected the parser module, got: {:?}",
        report.missing
    );
}

#[test]
fn a_single_type_file_whose_method_returns_a_value_is_still_reported() {
    // Arrange
    let root = unique_temp_dir("humble_adapter_returning_value");
    write_file(
        &root,
        "src/rtt_channel_config.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert_eq!(
        report.missing.len(),
        1,
        "a method producing a value must stay in scope, got: {:?}",
        report.missing
    );
}

#[test]
fn an_adapter_that_grows_a_branch_is_reported_again() {
    // Arrange
    let root = unique_temp_dir("humble_adapter_grown");
    write_file(
        &root,
        "src/board_halter.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert_eq!(
        report.missing.len(),
        1,
        "an adapter carrying a decision must be reported, got: {:?}",
        report.missing
    );
}

#[test]
fn analyze_packages_over_no_packages_returns_no_reports() {
    // Arrange
    let analyzer = Analyzer::new();

    // Act
    let reports = analyzer.analyze_packages(&[]).expect("analyze");

    // Assert
    assert!(reports.is_empty());
}

#[test]
fn analyze_packages_over_two_packages_returns_a_report_for_each() {
    // Arrange
    let first_root = unique_temp_dir("analyze_packages_first");
    let second_root = unique_temp_dir("analyze_packages_second");
    for root in [&first_root, &second_root] {
        fs::create_dir_all(root.join("src")).expect("create src");
        fs::write(root.join("src").join("lib.rs"), "pub fn f() {}\n").expect("write lib");
    }
    let packages = vec![
        package_context(&first_root, "first"),
        package_context(&second_root, "second"),
    ];
    let analyzer = Analyzer::new();

    // Act
    let reports = analyzer.analyze_packages(&packages).expect("analyze");

    // Assert
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].package_name, "first");
    assert_eq!(reports[1].package_name, "second");
}

#[test]
fn behavioral_validation_crate_source_is_still_ignored() {
    // Arrange
    let root = unique_temp_dir("validation_behavior_still_ignored");
    write_file(
        &root,
        "src/in_memory_transport.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "raft-validation"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "validation crate source should still be ignored even when behavioral, got: {:?}",
        report.missing
    );
}

#[test]
fn enum_with_non_struct_new_body_is_not_ignored() {
    // Arrange
    let root = unique_temp_dir("enum_non_struct_new");
    write_file(
        &root,
        "src/common_types/status.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/status.rs"),
        "enum with non-struct body in new() should not be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn generic_struct_with_new_is_ignored_as_trivial_constructor() {
    // Arrange
    let root = unique_temp_dir("generic_new_ignored");
    write_file(
        &root,
        "src/common_types/wrapper.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "struct with generic trivial new constructor should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn import_only_mod_file_is_ignored() {
    // Arrange
    let root = unique_temp_dir("import_only_mod_ignored");
    write_file(
        &root,
        "src/something/mod.rs",
        r#"
pub mod child;
use crate::foo::Bar;
pub use crate::baz::Qux;
"#,
    );
    write_file(
        &root,
        "src/something/child.rs",
        r#"
pub fn do_work() {}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    let missing_files = report
        .missing
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
    let root = unique_temp_dir("special_files_ignored");
    write_file(
        &root,
        "src/lib.rs",
        r#"
pub fn lib_entry() {}
"#,
    );
    write_file(
        &root,
        "src/main.rs",
        r#"
pub fn main_entry() {}
"#,
    );
    write_file(
        &root,
        "build.rs",
        r#"
fn main() {}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "lib.rs, main.rs, and build.rs should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn mirrored_test_path_is_detected_for_nested_non_ignored_source_file() {
    // Arrange
    let root = unique_temp_dir("mirrored_nested_path");
    write_file(
        &root,
        "src/implementations/raft/client.rs",
        r#"
pub fn start_client() {}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "raft-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.missing.iter().any(|gap| {
            gap.relative_source_file == "src/implementations/raft/client.rs"
                && gap.expected_test_file == "tests/implementations/raft/client_tests.rs"
        }),
        "expected nested mirrored path to be reported, got: {:?}",
        report.missing
    );
}

#[test]
fn mod_file_is_ignored_under_current_gate_policy() {
    // Arrange
    let root = unique_temp_dir("mod_file_ignored");
    write_file(
        &root,
        "src/feature/mod.rs",
        r#"
pub mod child;

pub fn configure() {}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        !report
            .missing
            .iter()
            .any(|gap| gap.relative_source_file == "src/feature/mod.rs"),
        "mod.rs files should be ignored under the current gate policy, got: {:?}",
        report.missing
    );
}

#[test]
fn multiple_definition_only_items_are_ignored() {
    // Arrange
    let root = unique_temp_dir("multiple_definition_only_ignored");
    write_file(
        &root,
        "src/common_types/types.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "multiple definition-only items should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn nontrivial_new_constructor_is_not_ignored() {
    // Arrange
    let root = unique_temp_dir("nontrivial_new_not_ignored");
    write_file(
        &root,
        "src/common_types/response.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/response.rs"),
        "nontrivial new constructor should not be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn pure_single_enum_definition_is_ignored() {
    // Arrange
    let root = unique_temp_dir("single_enum_ignored");
    write_file(
        &root,
        "src/state/storage/storage_query_result.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "single enum definition file should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn pure_single_struct_definition_is_ignored() {
    // Arrange
    let root = unique_temp_dir("single_struct_ignored");
    write_file(
        &root,
        "src/common_types/node_role.rs",
        r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRole {
    pub leader: bool,
}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "raft-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "single struct definition file should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn several_binary_entry_points_under_src_bin_are_all_ignored() {
    // Arrange
    let root = unique_temp_dir("src_bin_several_entry_points");
    write_file(
        &root,
        "src/bin/first.rs",
        r#"
fn main() {
    for argument in std::env::args() {
        println!("{argument}");
    }
}
"#,
    );
    write_file(
        &root,
        "src/bin/second.rs",
        r#"
fn main() {
    if cfg!(debug_assertions) {
        println!("debug");
    }
}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "every binary entry point under src/bin should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn single_enum_with_nontrivial_impl_is_not_ignored() {
    // Arrange
    let root = unique_temp_dir("nontrivial_impl_not_ignored");
    write_file(
        &root,
        "src/common_types/message_source.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/message_source.rs"),
        "enum with nontrivial impl should not be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn single_struct_with_constructor_not_named_new_is_not_ignored() {
    // Arrange
    let root = unique_temp_dir("ctor_not_new");
    write_file(
        &root,
        "src/common_types/builder.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/builder.rs"),
        "struct with constructor not named new should not be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn single_struct_with_empty_impl_is_not_ignored() {
    // Arrange
    let root = unique_temp_dir("empty_impl_not_ignored");
    write_file(
        &root,
        "src/common_types/marker.rs",
        r#"
pub struct Marker;

impl Marker {
}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        !report.is_empty(),
        "struct with empty impl should not be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn single_struct_with_multiple_impl_methods_is_not_ignored() {
    // Arrange
    let root = unique_temp_dir("multiple_methods_not_ignored");
    write_file(
        &root,
        "src/common_types/snapshot.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "raft-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/snapshot.rs"),
        "struct with multiple impl methods should not be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn single_struct_with_trait_impl_is_not_ignored() {
    // Arrange
    let root = unique_temp_dir("trait_impl_struct");
    write_file(
        &root,
        "src/common_types/convertible.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/convertible.rs"),
        "struct with trait impl should not be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn single_struct_with_trivial_new_constructor_is_ignored() {
    // Arrange
    let root = unique_temp_dir("trivial_constructor_ignored");
    write_file(
        &root,
        "src/implementations/recovery/peer_recovery_status.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "single struct with trivial new constructor should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn source_file_with_no_impl_block_is_definition_only() {
    // Arrange
    let root = unique_temp_dir("no_impl");
    write_file(
        &root,
        "src/common_types/point.rs",
        r#"
pub struct Point {
    pub x: f64,
    pub y: f64,
}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "struct with no impl should be ignored as definition-only, got: {:?}",
        report.missing
    );
}

#[test]
fn trait_and_type_definition_only_file_is_ignored() {
    // Arrange
    let root = unique_temp_dir("trait_type_only_ignored");
    write_file(
        &root,
        "src/common_types/storage_adapter.rs",
        r#"
pub trait StorageAdapter {
    type Query;
}

pub type StorageKey = u64;
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "trait/type definition-only file should be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn trait_impl_for_single_type_is_not_ignored() {
    // Arrange
    let root = unique_temp_dir("trait_impl_not_ignored");
    write_file(
        &root,
        "src/common_types/displayable.rs",
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
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-node"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report
            .missing
            .iter()
            .any(|gap| gap.relative_source_file == "src/common_types/displayable.rs"),
        "trait impl for a single type should not be ignored, got: {:?}",
        report.missing
    );
}

#[test]
fn validation_crate_src_files_are_ignored() {
    // Arrange
    let root = unique_temp_dir("validation_crate_ignored");
    write_file(
        &root,
        "src/protocol_cluster.rs",
        r#"
use std::collections::BTreeMap;

pub fn helper() -> usize {
    42
}
"#,
    );

    // Act
    let report = analyzer()
        .analyze_package(&package_context(&root, "demo-validation"))
        .expect("analysis should succeed");

    // Assert
    assert!(
        report.is_empty(),
        "validation crate src files should be ignored entirely, got: {:?}",
        report.missing
    );
}
