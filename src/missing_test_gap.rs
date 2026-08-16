// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingTestGap {
    pub package_name: String,
    pub relative_source_file: String,
    pub expected_test_file: String,
}
