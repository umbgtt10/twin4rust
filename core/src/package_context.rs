// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PackageContext {
    name: String,
    manifest_dir: PathBuf,
    source_roots: Vec<PathBuf>,
}

impl PackageContext {
    pub fn new(name: String, manifest_dir: PathBuf, source_roots: Vec<PathBuf>) -> Self {
        Self {
            name,
            manifest_dir,
            source_roots,
        }
    }

    pub fn manifest_dir(&self) -> &Path {
        &self.manifest_dir
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_roots(&self) -> &[PathBuf] {
        &self.source_roots
    }
}
