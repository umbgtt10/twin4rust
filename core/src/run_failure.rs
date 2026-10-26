// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use core::error;
use core::fmt;

use crate::parse_failure::ParseFailure;
use crate::workspace::source_read_failure::SourceReadFailure;
use crate::workspace::workspace_read_failure::WorkspaceReadFailure;

// Why a run produced no report, one variant per way it can fail. A failure the
// run only passes on says itself and keeps its own cause, so the chain a caller
// prints is the one the failure was raised with.
#[derive(Debug)]
pub enum RunFailure {
    NoParentDirectory,
    NoRootPackage,
    PackageNotFound(String),
    Parse(ParseFailure),
    SourceRead(SourceReadFailure),
    Workspace(WorkspaceReadFailure),
}

impl From<ParseFailure> for RunFailure {
    fn from(failure: ParseFailure) -> Self {
        Self::Parse(failure)
    }
}

impl From<SourceReadFailure> for RunFailure {
    fn from(failure: SourceReadFailure) -> Self {
        Self::SourceRead(failure)
    }
}

impl From<WorkspaceReadFailure> for RunFailure {
    fn from(failure: WorkspaceReadFailure) -> Self {
        Self::Workspace(failure)
    }
}

impl fmt::Display for RunFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoParentDirectory => {
                formatter.write_str("package manifest has no parent directory")
            }
            Self::NoRootPackage => {
                formatter.write_str("manifest contains multiple packages; pass --package <name>")
            }
            Self::PackageNotFound(name) => {
                write!(formatter, "package {name} was not found in the manifest")
            }
            Self::Parse(failure) => write!(formatter, "{failure}"),
            Self::SourceRead(failure) => write!(formatter, "{failure}"),
            Self::Workspace(failure) => write!(formatter, "{failure}"),
        }
    }
}

impl error::Error for RunFailure {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::NoParentDirectory | Self::NoRootPackage | Self::PackageNotFound(_) => None,
            Self::Parse(failure) => failure.source(),
            Self::SourceRead(failure) => failure.source(),
            Self::Workspace(failure) => failure.source(),
        }
    }
}
