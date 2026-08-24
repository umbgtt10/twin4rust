// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PackageContext {
    pub name: String,
    pub manifest_dir: PathBuf,
    pub source_roots: Vec<PathBuf>,
}
