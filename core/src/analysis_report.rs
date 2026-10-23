// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::missing_test_gap::MissingTestGap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisReport {
    package_name: String,
    missing: Vec<MissingTestGap>,
}

impl AnalysisReport {
    pub fn new(package_name: String, missing: Vec<MissingTestGap>) -> Self {
        Self {
            package_name,
            missing,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.missing.is_empty()
    }

    pub fn missing(&self) -> &[MissingTestGap] {
        &self.missing
    }

    pub fn missing_count(&self) -> usize {
        self.missing.len()
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }
}
