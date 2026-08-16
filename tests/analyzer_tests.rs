// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use twin4rust::analyzer::Analyzer;
use twin4rust::package_context::PackageContext;

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();

    let dir = std::env::temp_dir().join(format!("twin4rust_{label}_{nanos}"));
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

fn package_context(root: &Path, name: &str) -> PackageContext {
    PackageContext {
        name: name.to_string(),
        manifest_dir: root.to_path_buf(),
        source_roots: vec![root.join("src")],
    }
}

fn analyzer() -> Analyzer {
    Analyzer::new()
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
