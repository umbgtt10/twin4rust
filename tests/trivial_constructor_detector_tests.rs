// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;

use twin4rust::trivial_constructor_detector::TrivialConstructorDetector;

fn parse_items(source: &str) -> Vec<Item> {
    let file = syn::parse_file(source).expect("valid source");
    file.items
}

fn detector() -> TrivialConstructorDetector {
    TrivialConstructorDetector::new()
}

#[test]
fn empty_items_returns_false() {
    // Arrange
    let parsed = parse_items("");
    let items: Vec<&Item> = parsed.iter().collect();

    // Act
    let result = detector().file_has_trivial_constructor(&items);

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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(result);
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
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
    let result = detector().file_has_trivial_constructor(&items);

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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}

#[test]
fn struct_with_multiple_impls_returns_false() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}

#[test]
fn struct_with_constructor_not_named_new_returns_false() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}

#[test]
fn struct_with_non_struct_expr_body_returns_false() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
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
    let result = detector().file_has_trivial_constructor(&items);

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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(result);
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
    let result = detector().file_has_trivial_constructor(&items);

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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
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
    let result = detector().file_has_trivial_constructor(&items);

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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}

#[test]
fn impl_with_self_path_body_is_not_trivial() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}

// ---------------------------------------------------------------------------
// A second impl block alongside the trivial constructor
//
// `docs/RULES.md` rule 5 requires "exactly one inherent `impl` block" and "no
// other top-level behaviour", and its "what stays in scope" list names "a trait
// impl that carries at least one method" as reported. `is_allowed_item` waved
// through `Item::Impl(_)` unconditionally, so any second impl block escaped
// both conditions as long as one impl block was a trivial-`new`-only impl.
//
// The three tests above that look like they already cover this -- `struct_with_
// trait_impl_returns_false`, `struct_with_multiple_impls_returns_false`,
// `impl_on_wrong_target_name_returns_false` -- all pass for a different reason:
// none of them contains a `new` whose body is a struct-construction
// expression, so the constructor count never reaches one and the file is
// rejected before the extra impl is ever weighed.
// ---------------------------------------------------------------------------

#[test]
fn file_has_trivial_constructor_with_a_trait_impl_carrying_a_method_returns_false() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}

// The shape every adapter in a seam-heavy codebase takes: hold a collaborator,
// forward to it through the trait the seam is expressed as. The forwarded body
// is where the behaviour lives, and a trivial `new` beside it must not buy the
// file an exemption.
#[test]
fn file_has_trivial_constructor_with_a_forwarding_trait_impl_returns_false() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}

// Rule 5 says one inherent impl block, so a second one disqualifies even when
// its own method would have passed some other rule on its own.
#[test]
fn file_has_trivial_constructor_with_a_second_inherent_impl_returns_false() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}

// The boundary the fix must not cross. Rule 4 states in its own words that a
// trait impl carrying no methods "introduces no executable behaviour", so it is
// not other top-level behaviour and the exemption survives it.
#[test]
fn file_has_trivial_constructor_with_an_empty_trait_impl_returns_true() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(result);
}

// A top-level macro invocation was allowed here while both sibling detectors
// reject it, and rule 4 says an unexpanded macro "keeps the file in scope,
// whatever else the file contains". A macro can declare anything, including the
// behaviour the exemption claims is absent.
#[test]
fn file_has_trivial_constructor_with_a_top_level_macro_invocation_returns_false() {
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
    let result = detector().file_has_trivial_constructor(&items);

    // Assert
    assert!(!result);
}
