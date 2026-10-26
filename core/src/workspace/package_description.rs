// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::workspace::source_path::SourcePath;
use crate::workspace::target_description::TargetDescription;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDescription {
    pub name: String,
    pub manifest_path: SourcePath,
    pub targets: Vec<TargetDescription>,
}
