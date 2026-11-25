// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use quote::ToTokens;
use syn::ImplItem;
use syn::ImplItemFn;
use syn::ItemImpl;
use syn::parse_str;
use twin4rust::syntax::method_parts::MethodParts;

fn method(source: &str) -> ImplItemFn {
    let item_impl: ItemImpl =
        parse_str(&format!("impl Holder {{ {source} }}")).expect("valid impl");
    match item_impl.items.into_iter().next() {
        Some(ImplItem::Fn(method)) => method,
        _ => panic!("expected one method"),
    }
}

fn text_of<T: ToTokens>(tokens: Option<&T>) -> Option<String> {
    tokens.map(|tokens| tokens.to_token_stream().to_string())
}

#[test]
fn input_count_on_a_getter_returns_one() {
    // Arrange
    let method = method("pub fn id(&self) -> u32 { self.id }");

    // Act
    let result = MethodParts::new(&method).input_count();

    // Assert
    assert_eq!(result, 1);
}

#[test]
fn input_count_on_a_method_with_a_receiver_and_two_parameters_returns_three() {
    // Arrange
    let method = method("pub fn set(&mut self, id: u32, name: String) {}");

    // Act
    let result = MethodParts::new(&method).input_count();

    // Assert
    assert_eq!(result, 3);
}

#[test]
fn input_count_on_an_associated_function_without_parameters_returns_zero() {
    // Arrange
    let method = method("pub fn new() -> Self { Self }");

    // Act
    let result = MethodParts::new(&method).input_count();

    // Assert
    assert_eq!(result, 0);
}

#[test]
fn parameter_names_on_a_constructor_returns_each_name_in_order() {
    // Arrange
    let method = method("pub fn new(id: u32, name: String) -> Self { Self { id, name } }");

    // Act
    let result = MethodParts::new(&method).parameter_names();

    // Assert
    assert_eq!(result, vec![String::from("id"), String::from("name")]);
}

#[test]
fn parameter_names_on_a_method_with_a_receiver_returns_the_parameters_alone() {
    // Arrange
    let method = method("pub fn with_id(self, id: u32) -> Self { Self { id, ..self } }");

    // Act
    let result = MethodParts::new(&method).parameter_names();

    // Assert
    assert_eq!(result, vec![String::from("id")]);
}

#[test]
fn parameter_names_on_destructured_and_ignored_parameters_returns_the_named_ones_alone() {
    // Arrange
    let method =
        method("pub fn new((left, right): (u32, u32), _: u32, id: u32) -> Self { Self { id } }");

    // Act
    let result = MethodParts::new(&method).parameter_names();

    // Assert
    assert_eq!(result, vec![String::from("id")]);
}

#[test]
fn receiver_on_a_method_lending_self_returns_the_shared_receiver() {
    // Arrange
    let method = method("pub fn id(&self) -> u32 { self.id }");

    // Act
    let result = MethodParts::new(&method).receiver();

    // Assert
    assert_eq!(text_of(result), Some(String::from("& self")));
}

#[test]
fn receiver_on_a_method_taking_self_by_value_returns_the_owned_receiver() {
    // Arrange
    let method = method("pub fn with_id(self, id: u32) -> Self { Self { id, ..self } }");

    // Act
    let result = MethodParts::new(&method).receiver();

    // Assert
    assert_eq!(text_of(result), Some(String::from("self")));
}

#[test]
fn receiver_on_an_associated_function_returns_none() {
    // Arrange
    let method = method("pub fn new(id: u32) -> Self { Self { id } }");

    // Act
    let result = MethodParts::new(&method).receiver();

    // Assert
    assert!(result.is_none());
}

#[test]
fn single_expression_on_a_body_ending_in_a_semicolon_returns_none() {
    // Arrange
    let method = method("pub fn touch(&self) { self.id; }");

    // Act
    let result = MethodParts::new(&method).single_expression();

    // Assert
    assert!(result.is_none());
}

#[test]
fn single_expression_on_a_body_of_one_expression_returns_it() {
    // Arrange
    let method = method("pub fn id(&self) -> u32 { self.id }");

    // Act
    let result = MethodParts::new(&method).single_expression();

    // Assert
    assert_eq!(text_of(result), Some(String::from("self . id")));
}

#[test]
fn single_expression_on_a_body_of_two_statements_returns_none() {
    // Arrange
    let method = method("pub fn id(&self) -> u32 { let id = self.id; id }");

    // Act
    let result = MethodParts::new(&method).single_expression();

    // Assert
    assert!(result.is_none());
}

#[test]
fn single_expression_on_an_empty_body_returns_none() {
    // Arrange
    let method = method("pub fn nothing(&self) {}");

    // Act
    let result = MethodParts::new(&method).single_expression();

    // Assert
    assert!(result.is_none());
}
