// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use twin4rust::runner::Runner;

#[test]
fn runner_exists_and_is_send() {
    // Arrange & Act
    fn assert_send<T: Send>() {}
    assert_send::<Runner>();

    // Assert
    // Compile-time check: Runner is constructible and Send
}
