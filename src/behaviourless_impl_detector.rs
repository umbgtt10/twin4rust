// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::{ImplItem, Item};

#[derive(Default)]
pub struct BehaviourlessImplDetector;

impl BehaviourlessImplDetector {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // `impl Marker for T {}` and `impl<T> Alias for T where ...` introduce no
    // executable behaviour, so a mirrored test could only restate the compiler.
    // Only an impl that carries a method is worth a test.
    //
    // An inherent impl is not behaviourless even when empty: it is the block the
    // trivial-constructor and humble-adapter rules count, and treating a second
    // one as inert would let those rules admit a file they both describe as
    // holding exactly one.
    #[must_use]
    pub fn is_behaviourless_impl(&self, item: &Item) -> bool {
        match item {
            Item::Impl(item_impl) => {
                item_impl.trait_.is_some()
                    && !item_impl
                        .items
                        .iter()
                        .any(|member| matches!(member, ImplItem::Fn(_)))
            }
            _ => false,
        }
    }
}
