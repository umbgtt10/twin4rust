// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::workspace::source_path::SourcePath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetDescription {
    pub kinds: Vec<String>,
    pub src_path: SourcePath,
}
