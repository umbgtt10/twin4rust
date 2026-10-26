// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use core::error::Error;
use core::fmt;

// The workspace's description could not be read, and the reader's reason.
#[derive(Debug)]
pub struct WorkspaceReadFailure {
    pub cause: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for WorkspaceReadFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("failed to read Cargo metadata")
    }
}

impl Error for WorkspaceReadFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}
