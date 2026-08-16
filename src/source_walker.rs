// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub struct SourceWalker;

impl SourceWalker {
    pub fn walk(source_root: &Path) -> Vec<PathBuf> {
        WalkDir::new(source_root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "rs")
            })
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }
}
