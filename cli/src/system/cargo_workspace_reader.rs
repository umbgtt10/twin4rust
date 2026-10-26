// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use cargo_metadata::{Metadata, MetadataCommand, Package, Target};
use twin4rust::traits::workspace_reader::WorkspaceReader;
use twin4rust::workspace::package_description::PackageDescription;
use twin4rust::workspace::target_description::TargetDescription;
use twin4rust::workspace::workspace_description::WorkspaceDescription;
use twin4rust::workspace::workspace_read_failure::WorkspaceReadFailure;

use crate::system::portable_path::PortablePath;

// The workspace as `cargo metadata --no-deps` describes it, translated into the
// core's own terms so that nothing past this file knows cargo_metadata's types.
// The manifest path stays a `PathBuf` here and never enters the core, so a
// manifest path that is not UTF-8 works exactly as it always has; every path
// cargo reports back is UTF-8 by cargo_metadata's own types.
#[derive(Debug, Clone)]
pub struct CargoWorkspaceReader {
    manifest_path: Option<PathBuf>,
}

impl CargoWorkspaceReader {
    pub fn new(manifest_path: Option<PathBuf>) -> Self {
        Self { manifest_path }
    }

    pub fn translate(metadata: &Metadata) -> WorkspaceDescription {
        WorkspaceDescription {
            packages: metadata.packages.iter().map(Self::package).collect(),
            root_package: metadata
                .root_package()
                .map(|package| package.name.to_string()),
        }
    }

    fn package(package: &Package) -> PackageDescription {
        PackageDescription {
            name: package.name.to_string(),
            manifest_path: PortablePath::of_text(package.manifest_path.as_str()),
            targets: package.targets.iter().map(Self::target).collect(),
        }
    }

    fn target(target: &Target) -> TargetDescription {
        TargetDescription {
            kinds: target.kind.iter().map(ToString::to_string).collect(),
            src_path: PortablePath::of_text(target.src_path.as_str()),
        }
    }
}

impl WorkspaceReader for CargoWorkspaceReader {
    fn describe(&self) -> Result<WorkspaceDescription, WorkspaceReadFailure> {
        let mut command = MetadataCommand::new();
        command.no_deps();
        if let Some(manifest_path) = &self.manifest_path {
            command.manifest_path(manifest_path);
        }

        let metadata = command.exec().map_err(|error| WorkspaceReadFailure {
            cause: Box::new(error),
        })?;
        Ok(Self::translate(&metadata))
    }
}
