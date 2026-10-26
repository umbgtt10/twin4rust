// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::workspace::source_path::SourcePath;
use crate::workspace::source_read_failure::SourceReadFailure;

pub trait SourceTree: Sync {
    fn contains(&self, path: &SourcePath) -> bool;

    fn read_source(&self, file: &SourcePath) -> Result<String, SourceReadFailure>;

    fn rust_files(&self, root: &SourcePath) -> Result<Vec<SourcePath>, SourceReadFailure>;
}
