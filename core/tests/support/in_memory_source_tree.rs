// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use twin4rust::traits::source_tree::SourceTree;
use twin4rust::workspace::source_path::SourcePath;
use twin4rust::workspace::source_read_failure::SourceReadFailure;

// The source tree every core test reads, held in memory: a path either holds
// its contents or is listed and cannot be read. No test of the core touches a
// disk.
pub struct InMemorySourceTree {
    files: BTreeMap<SourcePath, Option<String>>,
}

impl InMemorySourceTree {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub fn with_file(mut self, path: &str, contents: &str) -> Self {
        self.files.insert(
            SourcePath::new(String::from(path)),
            Some(String::from(contents)),
        );
        self
    }

    pub fn with_unreadable_file(mut self, path: &str) -> Self {
        self.files.insert(SourcePath::new(String::from(path)), None);
        self
    }
}

impl Default for InMemorySourceTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceTree for InMemorySourceTree {
    fn contains(&self, path: &SourcePath) -> bool {
        self.files.contains_key(path)
    }

    fn read_source(&self, file: &SourcePath) -> Result<String, SourceReadFailure> {
        self.files
            .get(file)
            .cloned()
            .flatten()
            .ok_or_else(|| SourceReadFailure {
                path: file.clone(),
                cause: Box::new(NotHeld),
            })
    }

    fn rust_files(&self, root: &SourcePath) -> Result<Vec<SourcePath>, SourceReadFailure> {
        Ok(self
            .files
            .keys()
            .filter(|path| path.strip_prefix(root).is_some())
            .filter(|path| path.extension() == Some("rs"))
            .cloned()
            .collect())
    }
}

#[derive(Debug)]
pub struct NotHeld;

impl fmt::Display for NotHeld {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("not held")
    }
}

impl Error for NotHeld {}
