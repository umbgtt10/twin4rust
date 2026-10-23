// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    manifest_path: Option<PathBuf>,
    packages: Vec<String>,
}

impl Config {
    pub fn new(manifest_path: Option<PathBuf>, packages: Vec<String>) -> Self {
        Self {
            manifest_path,
            packages,
        }
    }

    pub fn manifest_path(&self) -> Option<&Path> {
        self.manifest_path.as_deref()
    }

    pub fn packages(&self) -> &[String] {
        &self.packages
    }
}
