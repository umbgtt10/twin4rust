// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

pub struct ConstructorScanState {
    type_name: Option<String>,
    constructor_impl_count: usize,
}

impl ConstructorScanState {
    pub fn new(type_name: Option<String>, constructor_impl_count: usize) -> Self {
        Self {
            type_name,
            constructor_impl_count,
        }
    }

    pub fn constructor_impl_count(&self) -> usize {
        self.constructor_impl_count
    }

    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }
}
