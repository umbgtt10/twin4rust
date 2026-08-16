// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use cargo_metadata::{Metadata, MetadataCommand, Package};

use crate::config::Config;
use crate::package_context::PackageContext;
use crate::target_root_collector::TargetRootCollector;

pub struct ManifestResolver {
    config: Config,
}

impl ManifestResolver {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn resolve(&self) -> Result<Vec<PackageContext>> {
        let mut command = MetadataCommand::new();
        command.no_deps();

        if let Some(manifest_path) = &self.config.manifest_path {
            command.manifest_path(manifest_path);
        }

        let metadata = command.exec().context("failed to read Cargo metadata")?;
        let packages = Self::select_packages(&metadata, &self.config.packages)?;

        packages
            .into_iter()
            .map(Self::build_package_context)
            .collect()
    }

    fn build_package_context(package: &Package) -> Result<PackageContext> {
        let manifest_dir = package
            .manifest_path
            .clone()
            .into_std_path_buf()
            .parent()
            .map(PathBuf::from)
            .context("package manifest has no parent directory")?;

        let mut root_collector = TargetRootCollector::new();
        root_collector.collect_from_targets(&package.targets);
        root_collector.ensure_fallback(&manifest_dir);
        let source_roots = root_collector.into_roots();
        let name = package.name.to_string();
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

    pub fn select_packages<'a>(
        metadata: &'a Metadata,
        requested: &[String],
    ) -> Result<Vec<&'a Package>> {
        if !requested.is_empty() {
            let mut selected = Vec::new();
            for package_name in requested {
                let package = metadata
                    .packages
                    .iter()
                    .find(|package| package.name == package_name)
                    .with_context(|| {
                        format!("package {package_name} was not found in the manifest")
                    })?;
                selected.push(package);
            }
            return Ok(selected);
        }

        if let Some(root) = metadata.root_package() {
            return Ok(vec![root]);
        }

        bail!("manifest contains multiple packages; pass --package <name>")
    }

    fn should_ignore_source_root(
        package_name: &str,
        manifest_dir: &Path,
        source_root: &Path,
    ) -> bool {
        package_name.ends_with("-validation") && source_root == manifest_dir.join("src")
    }

    pub fn relative_file(base_dir: &Path, file_path: &Path) -> String {
        file_path
            .strip_prefix(base_dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}
