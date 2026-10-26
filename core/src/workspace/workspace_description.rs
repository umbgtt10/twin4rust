// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::workspace::package_description::PackageDescription;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceDescription {
    pub packages: Vec<PackageDescription>,
    pub root_package: Option<String>,
}
