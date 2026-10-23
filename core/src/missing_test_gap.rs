// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingTestGap {
    package_name: String,
    relative_source_file: String,
    expected_test_file: String,
}

impl MissingTestGap {
    pub fn new(
        package_name: String,
        relative_source_file: String,
        expected_test_file: String,
    ) -> Self {
        Self {
            package_name,
            relative_source_file,
            expected_test_file,
        }
    }

    pub fn expected_test_file(&self) -> &str {
        &self.expected_test_file
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }

    pub fn relative_source_file(&self) -> &str {
        &self.relative_source_file
    }
}
