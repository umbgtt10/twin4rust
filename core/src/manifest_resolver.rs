// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::config::Config;
use crate::package_context::PackageContext;
use crate::run_failure::RunFailure;
use crate::target_root_collector::TargetRootCollector;
use crate::traits::workspace_reader::WorkspaceReader;
use crate::workspace::package_description::PackageDescription;
use crate::workspace::source_path::SourcePath;
use crate::workspace::workspace_description::WorkspaceDescription;

pub struct ManifestResolver<'a> {
    workspace_reader: &'a dyn WorkspaceReader,
}

impl<'a> ManifestResolver<'a> {
    pub fn new(workspace_reader: &'a dyn WorkspaceReader) -> Self {
        Self { workspace_reader }
    }

    pub fn relative_file(base_dir: &SourcePath, file_path: &SourcePath) -> String {
        file_path
            .strip_prefix(base_dir)
            .unwrap_or(file_path.as_str())
            .replace('\\', "/")
    }

    pub fn resolve(&self, config: &Config) -> Result<Vec<PackageContext>, RunFailure> {
        let workspace = self.workspace_reader.describe()?;
        let packages = Self::select_packages(&workspace, &config.packages)?;

        packages
            .into_iter()
            .map(Self::build_package_context)
            .collect()
    }

    pub fn select_packages<'w>(
        workspace: &'w WorkspaceDescription,
        requested: &[String],
    ) -> Result<Vec<&'w PackageDescription>, RunFailure> {
        if !requested.is_empty() {
            let mut selected = Vec::new();
            for package_name in requested {
                let package = workspace
                    .packages
                    .iter()
                    .find(|package| &package.name == package_name)
                    .ok_or_else(|| RunFailure::PackageNotFound(package_name.clone()))?;
                selected.push(package);
            }
            return Ok(selected);
        }

        workspace
            .root_package
            .as_ref()
            .and_then(|root| {
                workspace
                    .packages
                    .iter()
                    .find(|package| &package.name == root)
            })
            .map(|root| vec![root])
            .ok_or(RunFailure::NoRootPackage)
    }

    fn build_package_context(package: &PackageDescription) -> Result<PackageContext, RunFailure> {
        let manifest_dir = package
            .manifest_path
            .parent()
            .ok_or(RunFailure::NoParentDirectory)?;

        let mut root_collector = TargetRootCollector::new();
        root_collector.collect_from_targets(&package.targets);
        root_collector.ensure_fallback(&manifest_dir);
        let source_roots = root_collector.into_roots();
        let name = package.name.clone();
        let source_roots = source_roots
            .into_iter()
            .filter(|root| !Self::should_ignore_source_root(&name, &manifest_dir, root))
            .collect::<Vec<_>>();

        Ok(PackageContext {
            name,
            manifest_dir,
            source_roots,
        })
    }

    fn should_ignore_source_root(
        package_name: &str,
        manifest_dir: &SourcePath,
        source_root: &SourcePath,
    ) -> bool {
        package_name.ends_with("-validation") && *source_root == manifest_dir.join("src")
    }
}
