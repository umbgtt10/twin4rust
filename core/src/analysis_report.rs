// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::missing_test_gap::MissingTestGap;
use crate::unearned_mirror::UnearnedMirror;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisReport {
    package_name: String,
    missing: Vec<MissingTestGap>,
    unearned: Vec<UnearnedMirror>,
}

impl AnalysisReport {
    pub fn new(
        package_name: String,
        missing: Vec<MissingTestGap>,
        unearned: Vec<UnearnedMirror>,
    ) -> Self {
        Self {
            package_name,
            missing,
            unearned,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.missing.is_empty() && self.unearned.is_empty()
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

    pub fn unearned(&self) -> &[UnearnedMirror] {
        &self.unearned
    }

    pub fn unearned_count(&self) -> usize {
        self.unearned.len()
    }
}
