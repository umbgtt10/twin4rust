// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::parse_file;
use syn::parse_str;
use syn::{Item, ItemImpl};
use twin4rust::data_holder_detector::DataHolderDetector;

fn detector() -> DataHolderDetector {
    DataHolderDetector::new()
}

fn parse_items(source: &str) -> Vec<Item> {
    let file = parse_file(source).expect("valid source");
    file.items
}

#[test]
fn empty_items_returns_false() {
    // Arrange
    let parsed = parse_items("");
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

// The shape every adapter in a seam-heavy codebase takes: hold a collaborator,
// forward to it through the trait the seam is expressed as. The forwarded body
// is where the behaviour lives, and accessors beside it must not buy the file
// an exemption.
#[test]
fn file_is_data_holder_with_a_forwarding_trait_impl_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
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
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

// A record may spread its accessors over more than one inherent impl block --
// stern4rust counts a type's methods across every block -- so a getter in a
// second one keeps the exemption. What a block holds decides, not how many.
#[test]
fn file_is_data_holder_with_a_getter_in_a_second_inherent_impl_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Worker {
    id: u32,
}

impl Worker {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

impl Worker {
    pub fn id(&self) -> u32 {
        self.id
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

#[test]
fn file_is_data_holder_with_a_getter_only_record_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Snapshot {
    height: u64,
}
impl Snapshot {
    pub fn height(&self) -> u64 {
        self.height
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

#[test]
fn file_is_data_holder_with_a_new_calling_a_helper_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Seeded {
    pub seed: u64,
}
impl Seeded {
    pub fn new(seed: u64) -> Self {
        Self { seed: mix(seed) }
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn file_is_data_holder_with_a_new_converting_its_parameter_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Named {
    pub name: String,
}
impl Named {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn file_is_data_holder_with_a_step_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Options {
    pub depth: u32,
}
impl Options {
    pub fn with_depth(self, depth: u32) -> Self {
        Self { depth, ..self }
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

// A top-level macro invocation was allowed here while both sibling detectors
// reject it, and rule 4 says an unexpanded macro "keeps the file in scope,
// whatever else the file contains". A macro can declare anything, including the
// behaviour the exemption claims is absent.
#[test]
fn file_is_data_holder_with_a_top_level_macro_invocation_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Registry {
    value: u32,
}

impl Registry {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

register_handlers!(Registry);
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn file_is_data_holder_with_a_trait_impl_carrying_a_method_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Wrapper {
    value: u32,
}

impl Wrapper {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn file_is_data_holder_with_accessors_across_impls_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Window {
    pub start: u64,
    pub end: u64,
}
impl Window {
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }
}
impl Window {
    pub fn with_end(self, end: u64) -> Self {
        Self { end, ..self }
    }
    pub fn start(&self) -> u64 {
        self.start
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

// The boundary the fix must not cross. Rule 4 states in its own words that a
// trait impl carrying no methods "introduces no executable behaviour", so it is
// not other top-level behaviour and the exemption survives it.
#[test]
fn file_is_data_holder_with_an_empty_trait_impl_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Marker {
    value: u32,
}

impl Marker {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl Send for Marker {}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

#[test]
fn foreign_mod_item_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Foo;

impl Foo {
    pub fn new() -> Self {
        Self
    }
}

extern "C" {
    fn external();
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn impl_before_struct_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
impl Foo {
    pub fn new() -> Self {
        Self
    }
}

pub struct Foo;
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn impl_holds_only_accessors_for_a_different_type_name_returns_false() {
    // Arrange
    let item: ItemImpl =
        parse_str("impl Holder { pub fn new() -> Self { Self {} } }").expect("parse impl");
    let detector = DataHolderDetector::new();

    // Act & Assert
    assert!(!detector.impl_holds_only_accessors(&item, "Other"));
}

#[test]
fn impl_holds_only_accessors_on_a_bare_new_returns_true() {
    // Arrange
    let item: ItemImpl =
        parse_str("impl Holder { pub fn new() -> Self { Self {} } }").expect("parse impl");
    let detector = DataHolderDetector::new();

    // Act & Assert
    assert!(detector.impl_holds_only_accessors(&item, "Holder"));
}

#[test]
fn impl_holds_only_accessors_on_a_new_that_does_work_returns_false() {
    // Arrange -- the body must be a single struct expression. A `new` that
    // computes anything, even to build the same type, is not trivial.
    let item: ItemImpl =
        parse_str("impl Holder { pub fn new() -> Self { let seed = compute(); Self { seed } } }")
            .expect("parse impl");
    let detector = DataHolderDetector::new();

    // Act & Assert
    assert!(!detector.impl_holds_only_accessors(&item, "Holder"));
}

#[test]
fn impl_holds_only_accessors_on_a_trait_impl_returns_false() {
    // Arrange
    let item: ItemImpl = parse_str("impl Default for Holder { fn default() -> Self { Self {} } }")
        .expect("parse impl");
    let detector = DataHolderDetector::new();

    // Act & Assert
    assert!(!detector.impl_holds_only_accessors(&item, "Holder"));
}

#[test]
fn impl_holds_only_accessors_on_an_impl_with_another_method_returns_false() {
    // Arrange
    let item: ItemImpl =
        parse_str("impl Holder { pub fn new() -> Self { Self } pub fn work(&self) -> u32 { 1 } }")
            .expect("parse impl");
    let detector = DataHolderDetector::new();

    // Act & Assert
    assert!(!detector.impl_holds_only_accessors(&item, "Holder"));
}

#[test]
fn impl_on_wrong_target_name_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Foo;

impl Bar {
    pub fn new() -> Self {
        Self
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn impl_with_self_path_body_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Wrapper;

impl Wrapper {
    pub fn new() -> Self {
        Self
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

#[test]
fn multiple_structs_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Foo;
pub struct Bar;
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn single_enum_with_non_struct_new_body_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub enum Status {
    Active,
}

impl Status {
    pub fn new() -> Self {
        Self::Active
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn single_struct_with_trivial_new_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Foo {
    pub value: u32,
}

impl Foo {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

#[test]
fn struct_with_a_unit_path_body_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Counter;

impl Counter {
    pub fn new() -> Self {
        Counter
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

#[test]
fn struct_with_bare_function_item_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Foo;

pub fn helper() -> u32 {
    42
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn struct_with_constructor_not_named_new_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Builder;

impl Builder {
    pub fn build() -> Self {
        Builder
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

#[test]
fn struct_with_generic_new_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Wrapper;

impl Wrapper {
    pub fn new<T>(value: T) -> Self {
        Self
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn struct_with_multiple_impls_of_plain_constructors_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Worker;

impl Worker {
    pub fn new() -> Self {
        Self
    }
}

impl Worker {
    pub fn spawn() -> Self {
        Self
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}

#[test]
fn struct_with_trait_impl_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Wrapper;

impl From<u64> for Wrapper {
    fn from(v: u64) -> Self {
        Wrapper
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn struct_with_two_stmt_body_in_new_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Foo;

impl Foo {
    pub fn new() -> Self {
        let x = Self;
        x
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn struct_with_wrong_return_type_in_new_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Foo;

impl Foo {
    pub fn new() -> u32 {
        42
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn struct_without_constructor_returns_false() {
    // Arrange
    let parsed = parse_items(
        r#"
pub struct Empty;
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(!result);
}

#[test]
fn use_and_struct_and_impl_returns_true() {
    // Arrange
    let parsed = parse_items(
        r#"
use std::fmt;

pub struct Point {
    pub x: f64,
}

impl Point {
    pub fn new(x: f64) -> Self {
        Self { x }
    }
}
"#,
    );
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_is_data_holder(&items);

    // Assert
    assert!(result);
}
