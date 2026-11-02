// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The methods a record may carry and stay a record: a getter reading one field,
// a plain constructor of bare parameters, a `with_` step replacing one field.
// They are stern4rust's shapes for `records-and-objects`, so the two tools agree
// on which types have nothing to test.

use syn::ImplItem;
use syn::ImplItemFn;
use syn::ItemImpl;
use syn::parse_str;
use twin4rust::accessor_classifier::AccessorClassifier;

fn classifier() -> AccessorClassifier {
    AccessorClassifier::new()
}

fn method(source: &str) -> ImplItemFn {
    let item_impl: ItemImpl =
        parse_str(&format!("impl Holder {{ {source} }}")).expect("valid impl");
    match item_impl.items.into_iter().next() {
        Some(ImplItem::Fn(method)) => method,
        _ => panic!("expected one method"),
    }
}

#[test]
fn is_accessor_on_a_build_step_returns_false() {
    // Arrange
    let method = method("pub fn build(self) -> Widget { Widget { id: self.id } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_constructor_naming_its_type_returns_true() {
    // Arrange
    let method =
        method("pub fn from_parts(id: u32, name: String) -> Holder { Holder { id, name } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_constructor_of_another_type_returns_false() {
    // Arrange
    let method = method("pub fn new(id: u32) -> Other { Other { id } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_constructor_with_a_computed_field_returns_false() {
    // Arrange
    let method = method("pub fn new(id: u32) -> Self { Self { id: id + 1 } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_constructor_with_a_rest_returns_false() {
    // Arrange
    let method = method("pub fn new(id: u32) -> Self { Self { id, ..Default::default() } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_constructor_with_two_statements_returns_false() {
    // Arrange
    let method = method("pub fn new() -> Self { let id = 1; Self { id } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_generic_plain_constructor_returns_true() {
    // Arrange
    let method = method("pub fn new<T>(value: T) -> Self { Self { value } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_getter_cloning_a_field_returns_true() {
    // Arrange
    let method = method("pub fn name(&self) -> String { self.name.clone() }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_getter_lending_a_field_returns_true() {
    // Arrange
    let method = method("pub fn name(&self) -> &String { &self.name }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_getter_reading_a_field_returns_true() {
    // Arrange
    let method = method("pub fn id(&self) -> u32 { self.id }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_getter_through_as_str_returns_true() {
    // Arrange
    let method = method("pub fn name(&self) -> &str { self.name.as_str() }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_getter_with_a_mutable_receiver_returns_false() {
    // Arrange
    let method = method("pub fn id(&mut self) -> u32 { self.id }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_getter_with_a_parameter_returns_false() {
    // Arrange
    let method = method("pub fn id(&self, offset: u32) -> u32 { self.id }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_method_computing_from_fields_returns_false() {
    // Arrange
    let method = method("pub fn total(&self) -> u32 { self.a + self.b }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_method_returning_a_variant_returns_false() {
    // Arrange
    let method = method("pub fn new() -> Self { Self::Active }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_mutating_method_returns_false() {
    // Arrange
    let method = method("pub fn reset(&mut self) { self.id = 0; }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_plain_constructor_returns_true() {
    // Arrange
    let method = method("pub fn new(id: u32) -> Self { Self { id } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_step_by_reference_returns_false() {
    // Arrange
    let method = method("pub fn with_id(&self, id: u32) -> Self { Self { id, ..*self } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_step_returns_true() {
    // Arrange
    let method = method("pub fn with_id(self, id: u32) -> Self { Self { id, ..self } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_step_setting_two_fields_returns_false() {
    // Arrange
    let method = method("pub fn with_both(self, a: u32, b: u32) -> Self { Self { a, b, ..self } }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(!result);
}

#[test]
fn is_accessor_on_a_tuple_constructor_returns_true() {
    // Arrange
    let method = method("pub fn new(id: u32) -> Self { Self(id) }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}

#[test]
fn is_accessor_on_a_unit_constructor_returns_true() {
    // Arrange
    let method = method("pub fn empty() -> Self { Self }");

    // Act
    let result = classifier().is_accessor(&method, "Holder");

    // Assert
    assert!(result);
}
