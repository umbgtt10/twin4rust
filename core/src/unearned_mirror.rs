// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnearnedMirror {
    pub package_name: String,
    pub relative_source_file: String,
    pub mirror_test_file: String,
}
