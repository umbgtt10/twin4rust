// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Mirrors src/constructor_scan_state.rs.

use twin4rust::constructor_scan_state::ConstructorScanState;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let state = ConstructorScanState::new(Some(String::from("Widget")), 2);

    // Assert
    assert_eq!(state.type_name(), Some("Widget"));
    assert_eq!(state.constructor_impl_count(), 2);
}
