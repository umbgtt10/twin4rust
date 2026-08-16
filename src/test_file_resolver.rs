// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct TestFileResolver;

impl TestFileResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn expected_test_file(&self, manifest_dir: &Path, source_file: &Path) -> Option<PathBuf> {
        let src_root = manifest_dir.join("src");
        let tests_root = manifest_dir.join("tests");

        let relative_source = source_file.strip_prefix(&src_root).ok()?;
        let file_name = relative_source.file_name()?.to_string_lossy();

        if file_name == "lib.rs" || file_name == "main.rs" || file_name == "mod.rs" {
            return None;
        }

        let base_name = relative_source.file_stem()?.to_string_lossy();
        let test_file_name = format!("{base_name}_tests.rs");

        let parent = relative_source.parent();
        match parent {
            Some(parent) if !parent.as_os_str().is_empty() => {
                Some(tests_root.join(parent).join(test_file_name))
            }
            _ => Some(tests_root.join(test_file_name)),
        }
    }
}
