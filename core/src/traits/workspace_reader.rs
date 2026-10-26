// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::workspace::workspace_description::WorkspaceDescription;
use crate::workspace::workspace_read_failure::WorkspaceReadFailure;

pub trait WorkspaceReader: Sync {
    fn describe(&self) -> Result<WorkspaceDescription, WorkspaceReadFailure>;
}
