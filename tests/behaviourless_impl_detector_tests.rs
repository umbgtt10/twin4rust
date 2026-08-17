// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Which impl blocks carry no executable behaviour. Both the definition-only rule
// and the trivial-constructor rule need this answer, and before it lived here
// only the first of them had it -- the second waved every impl block through,
// which is what let an adapter with a real trait impl escape the gate on the
// strength of holding a `new`.

use syn::Item;

use twin4rust::behaviourless_impl_detector::BehaviourlessImplDetector;

fn parse_first_item(source: &str) -> Item {
    let file = syn::parse_file(source).expect("valid source");
    file.items.into_iter().next().expect("one item")
}

fn detector() -> BehaviourlessImplDetector {
    BehaviourlessImplDetector::new()
}

#[test]
fn is_behaviourless_impl_of_a_marker_trait_impl_returns_true() {
    // Arrange
    let item = parse_first_item("impl Marker for Wrapper {}");

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(result);
}

#[test]
fn is_behaviourless_impl_of_a_blanket_trait_impl_with_a_where_clause_returns_true() {
    // Arrange
    let item = parse_first_item("impl<T> Alias for T where T: Send {}");

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(result);
}

// An associated type or const is a definition, not a body, so an impl holding
// nothing else still has nothing to call.
#[test]
fn is_behaviourless_impl_of_a_trait_impl_with_only_associated_items_returns_true() {
    // Arrange
    let item = parse_first_item(
        r#"
impl Storage for Disk {
    type Key = u64;
    const LIMIT: usize = 8;
}
"#,
    );

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(result);
}

#[test]
fn is_behaviourless_impl_of_a_trait_impl_carrying_a_method_returns_false() {
    // Arrange
    let item = parse_first_item(
        r#"
impl Display for Wrapper {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.value)
    }
}
"#,
    );

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(!result);
}

// One method among associated items is still a method.
#[test]
fn is_behaviourless_impl_of_a_trait_impl_mixing_a_method_with_associated_items_returns_false() {
    // Arrange
    let item = parse_first_item(
        r#"
impl Storage for Disk {
    type Key = u64;

    fn read(&self) -> u64 {
        0
    }
}
"#,
    );

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(!result);
}

// The deliberate asymmetry. An inherent impl is the block the trivial-constructor
// and humble-adapter rules count, so calling an empty one inert would let those
// rules admit a second one and still claim the file holds exactly one.
#[test]
fn is_behaviourless_impl_of_an_empty_inherent_impl_returns_false() {
    // Arrange
    let item = parse_first_item("impl Wrapper {}");

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(!result);
}

#[test]
fn is_behaviourless_impl_of_an_inherent_impl_carrying_a_method_returns_false() {
    // Arrange
    let item = parse_first_item(
        r#"
impl Wrapper {
    pub fn value(&self) -> u32 {
        self.value
    }
}
"#,
    );

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(!result);
}

#[test]
fn is_behaviourless_impl_of_an_item_that_is_not_an_impl_returns_false() {
    // Arrange
    let item = parse_first_item("pub struct Wrapper { value: u32 }");

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(!result);
}

#[test]
fn is_behaviourless_impl_of_a_trait_definition_returns_false() {
    // Arrange
    let item = parse_first_item("pub trait Marker {}");

    // Act
    let result = detector().is_behaviourless_impl(&item);

    // Assert
    assert!(!result);
}
