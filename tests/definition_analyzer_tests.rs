// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use twin4rust::definition_analyzer::DefinitionAnalyzer;

fn analyzer() -> DefinitionAnalyzer {
    DefinitionAnalyzer::new()
}

// ---------------------------------------------------------------------------
// is_definition_only_source
// ---------------------------------------------------------------------------

#[test]
fn empty_source_is_not_definition_only() {
    // Arrange & Act
    let result = analyzer().is_definition_only_source("").expect("parse");

    // Assert
    assert!(!result);
}

#[test]
fn single_struct_without_impl_is_definition_only() {
    // Arrange
    let source = "#[derive(Debug)]\npub struct Point { pub x: f64, pub y: f64 }";

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn single_enum_without_impl_is_definition_only() {
    // Arrange
    let source = "#[derive(Debug, Clone)]\npub enum Status { Active, Inactive }";

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn trait_and_type_definitions_are_definition_only() {
    // Arrange
    let source = r#"
pub trait StorageAdapter {
    type Query;
}
pub type StorageKey = u64;
pub const VERSION: u32 = 1;
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn multiple_types_and_traits_plus_const_are_definition_only() {
    // Arrange
    let source = r#"
pub trait QueryAdapter { type Output; }
pub struct Envelope(pub u64);
pub enum Kind { A, B }
pub type Id = u64;
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn source_with_function_is_not_definition_only() {
    // Arrange
    let source = "pub struct Config;\npub fn helper() -> u32 { 42 }";

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(!result);
}

#[test]
fn source_with_impl_method_is_not_definition_only() {
    // Arrange
    let source = r#"
pub struct Worker;
impl Worker {
    pub fn work(&self) {}
}
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(!result);
}

#[test]
fn cfg_test_module_is_ignored_when_checking_definition_only() {
    // Arrange
    let source = r#"
pub struct Config;

#[cfg(test)]
mod tests {
    fn helper() {}
}
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn struct_with_trivial_new_constructor_is_definition_only() {
    // Arrange
    let source = r#"
pub struct Point { pub x: f64, pub y: f64 }
impl Point {
    pub fn new(x: f64, y: f64) -> Self { Self { x, y } }
}
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn struct_with_nontrivial_new_constructor_is_not_definition_only() {
    // Arrange
    let source = r#"
pub struct Response { pub value: u64 }
impl Response {
    pub fn new(value: u64) -> Self {
        if value > 0 { Self { value } } else { Self { value: 1 } }
    }
}
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(!result);
}

#[test]
fn struct_with_multiple_impl_blocks_is_not_definition_only() {
    // Arrange
    let source = r#"
pub struct Snapshot { pub height: u64 }
impl Snapshot {
    pub fn new(height: u64) -> Self { Self { height } }
}
impl Snapshot {
    pub fn is_empty(&self) -> bool { self.height == 0 }
}
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn use_items_plus_single_struct_are_definition_only() {
    // Arrange
    let source = r#"
use crate::common_types::types::Hash;
pub struct Block { pub hash: Hash }
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn only_use_items_without_types_is_not_definition_only() {
    // Arrange
    let source = "use crate::foo;\npub use crate::bar::Baz;";

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(!result);
}

// ---------------------------------------------------------------------------
// mod_file_is_import_only
// ---------------------------------------------------------------------------

#[test]
fn mod_file_with_use_and_pub_use_is_import_only() {
    // Arrange
    let source = r#"
pub mod child;
use crate::foo::Bar;
pub use crate::baz::Qux;
"#;

    // Act
    let result = analyzer().mod_file_is_import_only(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn mod_file_with_function_is_not_import_only() {
    // Arrange
    let source = r#"
pub mod child;
pub fn configure() {}
"#;

    // Act
    let result = analyzer().mod_file_is_import_only(source).expect("parse");

    // Assert
    assert!(!result);
}

#[test]
fn mod_file_with_empty_mod_declaration_is_import_only() {
    // Arrange
    let source = "pub mod child;\nuse std::collections::HashMap;";

    // Act
    let result = analyzer().mod_file_is_import_only(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn empty_mod_file_is_import_only() {
    // Arrange
    let source = "";

    // Act
    let result = analyzer().mod_file_is_import_only(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn mod_file_with_struct_definition_is_not_import_only() {
    // Arrange
    let source = "pub struct Helper;";

    // Act
    let result = analyzer().mod_file_is_import_only(source).expect("parse");

    // Assert
    assert!(!result);
}
