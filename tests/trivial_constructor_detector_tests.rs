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
