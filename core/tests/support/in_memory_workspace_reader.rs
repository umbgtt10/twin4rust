// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use twin4rust::traits::workspace_reader::WorkspaceReader;
use twin4rust::workspace::package_description::PackageDescription;
use twin4rust::workspace::source_path::SourcePath;
use twin4rust::workspace::target_description::TargetDescription;
use twin4rust::workspace::workspace_description::WorkspaceDescription;
use twin4rust::workspace::workspace_read_failure::WorkspaceReadFailure;

// The workspace every core test resolves against, handed over whole rather than
// read from cargo.
pub struct InMemoryWorkspaceReader {
    workspace: Option<WorkspaceDescription>,
}

impl InMemoryWorkspaceReader {
    pub fn new(workspace: WorkspaceDescription) -> Self {
        Self {
            workspace: Some(workspace),
        }
    }

    pub fn failing() -> Self {
        Self { workspace: None }
    }
}

impl WorkspaceReader for InMemoryWorkspaceReader {
    fn describe(&self) -> Result<WorkspaceDescription, WorkspaceReadFailure> {
        self.workspace.clone().ok_or_else(|| WorkspaceReadFailure {
            cause: Box::new(NotHeld),
        })
    }
}

pub fn library_package(name: &str, manifest_dir: &str) -> PackageDescription {
    let dir = SourcePath::new(String::from(manifest_dir));
    PackageDescription {
        name: String::from(name),
        manifest_path: dir.join("Cargo.toml"),
        targets: vec![TargetDescription {
            kinds: vec![String::from("lib")],
            src_path: dir.join("src").join("lib.rs"),
        }],
    }
}
