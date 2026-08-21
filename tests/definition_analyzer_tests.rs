// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Attribute;
use syn::ItemMod;
use syn::parse_str;
use twin4rust::definition_analyzer::DefinitionAnalyzer;

fn analyzer() -> DefinitionAnalyzer {
    DefinitionAnalyzer::new()
}

#[test]
fn a_macro_invocation_alongside_a_const_is_not_definition_only() {
    // Arrange -- the const must not buy the macro a free pass. One opaque item
    // is enough to keep the whole file in scope.
    let source = r#"
pub const VERSION: u32 = 1;
tonic::include_proto!("consensus");
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
fn empty_mod_file_is_import_only() {
    // Arrange
    let source = "";

    // Act
    let result = analyzer().mod_file_is_import_only(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn empty_source_is_not_definition_only() {
    // Arrange & Act
    let result = analyzer().is_definition_only_source("").expect("parse");

    // Assert
    assert!(!result);
}

#[test]
fn has_test_attrs_on_a_cfg_test_attribute_returns_true() {
    // Arrange
    let item: ItemMod = parse_str("#[cfg(test)] mod tests {}").expect("parse module");

    // Act & Assert
    assert!(DefinitionAnalyzer::has_test_attrs(&item.attrs));
}

#[test]
fn has_test_attrs_on_a_cfg_that_is_not_test_returns_false() {
    // Arrange
    let item: ItemMod = parse_str("#[cfg(feature = \"demo\")] mod demo {}").expect("parse module");

    // Act & Assert
    assert!(!DefinitionAnalyzer::has_test_attrs(&item.attrs));
}

#[test]
fn has_test_attrs_on_no_attributes_returns_false() {
    // Arrange
    let attributes: Vec<Attribute> = vec![];

    // Act & Assert
    assert!(!DefinitionAnalyzer::has_test_attrs(&attributes));
}

#[test]
fn is_definition_only_source_with_a_trivial_new_and_a_behaviour_bearing_trait_impl_is_not_definition_only()
 {
    // Arrange
    let source = r#"
use crate::assembly::ei_notify::EiNotify;

pub struct ChannelEiNotify<const N: usize> {
    receiver: Receiver<'static, CriticalSectionRawMutex, (), N>,
}

impl<const N: usize> ChannelEiNotify<N> {
    pub fn new(receiver: Receiver<'static, CriticalSectionRawMutex, (), N>) -> Self {
        Self { receiver }
    }
}

impl<const N: usize> EiNotify for ChannelEiNotify<N> {
    async fn receive(&self) {
        self.receiver.receive().await
    }
}
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(!result);
}

// An empty trait impl beside the constructor keeps the exemption: rule 4 already
// calls a method-free trait impl inert, and rule 5 must agree with it.
#[test]
fn is_definition_only_source_with_a_trivial_new_and_an_empty_trait_impl_is_definition_only() {
    // Arrange
    let source = r#"
pub struct Marker {
    value: u32,
}

impl Marker {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl Send for Marker {}
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
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
fn mod_file_with_struct_definition_is_not_import_only() {
    // Arrange
    let source = "pub struct Helper;";

    // Act
    let result = analyzer().mod_file_is_import_only(source).expect("parse");

    // Assert
    assert!(!result);
}

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
fn only_a_macro_invocation_is_not_definition_only() {
    // Arrange -- a top-level macro expands to code this analyzer never sees,
    // so its unexpanded form says nothing about whether the file carries
    // behaviour. Excluding it would under-report, which is the failure mode
    // that matters for a gate: a silent pass.
    let source = "lazy_static! { static ref REGISTRY: Registry = Registry::new(); }";

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(!result);
}

#[test]
fn only_const_items_is_definition_only() {
    // Arrange -- a module of lookup tables declares data and nothing else.
    // There is no behaviour a mirrored test could assert, so demanding one
    // produces a stub written purely to quieten the gate.
    let source = r#"
pub const KNOWN_FOREIGN_TRAITS: &[&str] = &["Display", "Debug", "Clone"];
pub const STD_CONSTRUCTORS: &[&str] = &["Box", "Arc", "Rc"];
"#;

    // Act
    let result = analyzer().is_definition_only_source(source).expect("parse");

    // Assert
    assert!(result);
}

#[test]
fn only_static_items_is_definition_only() {
    // Arrange -- `static` is the same shape of declaration as `const` and must
    // not be treated differently just because it has a storage location.
    let source = "pub static BANNER: &str = \"twin4rust\";";

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
fn single_struct_without_impl_is_definition_only() {
    // Arrange
    let source = "#[derive(Debug)]\npub struct Point { pub x: f64, pub y: f64 }";

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
// This asserted `result` while its name said the opposite, pinning the defect
// rather than the rule: the second impl block carries `is_empty`, and rule 5
// asks for exactly one inherent impl and no other top-level behaviour, so the
// file is reported. The name was right and the assertion was not.
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
    assert!(!result);
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
fn use_items_alongside_consts_is_definition_only() {
    // Arrange -- imports do not make a data module behavioural. What decides it
    // is that something inert is declared, not that nothing is imported.
    let source = r#"
use std::time::Duration;
pub const TIMEOUT: Duration = Duration::from_secs(30);
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
