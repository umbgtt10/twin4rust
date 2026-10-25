// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

pub struct ConstructorScanState {
    pub type_name: Option<String>,
    pub constructor_impl_count: usize,
}
