// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::analysis_report::AnalysisReport;
use crate::definition_analyzer::DefinitionAnalyzer;
use crate::manifest_resolver::ManifestResolver;
use crate::missing_test_gap::MissingTestGap;
use crate::package_context::PackageContext;
use crate::source_walker::SourceWalker;
use crate::test_file_resolver::TestFileResolver;

#[derive(Default)]
pub struct Analyzer {
    definition_analyzer: DefinitionAnalyzer,
    test_file_resolver: TestFileResolver,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            definition_analyzer: DefinitionAnalyzer::new(),
            test_file_resolver: TestFileResolver::new(),
        }
    }

    pub fn analyze_packages(&self, packages: &[PackageContext]) -> Result<Vec<AnalysisReport>> {
        packages
            .iter()
            .map(|pkg| self.analyze_package(pkg))
            .collect()
    }

    pub fn analyze_package(&self, package: &PackageContext) -> Result<AnalysisReport> {
        let mut missing = Vec::new();
        for source_root in &package.source_roots {
            missing.extend(self.process_source_root(source_root, package)?);
        }
        missing.sort_by(|left, right| {
            left.package_name
                .cmp(&right.package_name)
                .then_with(|| left.relative_source_file.cmp(&right.relative_source_file))
        });
        Ok(AnalysisReport::new(package.name.clone(), missing))
    }

    fn process_source_root(
        &self,
        source_root: &Path,
        package: &PackageContext,
    ) -> Result<Vec<MissingTestGap>> {
        if !source_root.exists() {
            return Ok(Vec::new());
        }
        let mut missing = Vec::new();
        for source_path in SourceWalker::walk(source_root) {
            if let Some(gap) = self.check_source_file(&source_path, package)? {
                missing.push(gap);
            }
        }
        Ok(missing)
    }

    fn check_source_file(
        &self,
        source_path: &Path,
        package: &PackageContext,
    ) -> Result<Option<MissingTestGap>> {
        let relative_source = ManifestResolver::relative_file(&package.manifest_dir, source_path);

        if self.should_ignore_package_source(package, &relative_source) {
            return Ok(None);
        }

        let source = fs::read_to_string(source_path)
            .with_context(|| format!("failed to read source file {}", source_path.display()))?;

        if self.should_ignore_source_file(&relative_source, &source)? {
            return Ok(None);
        }

        if self
            .definition_analyzer
            .is_definition_only_source(&source)?
        {
            return Ok(None);
        }

        let expected = self
            .test_file_resolver
            .expected_test_file(&package.manifest_dir, source_path);
        let Some(expected_test) = expected else {
            return Ok(None);
        };

        if expected_test.exists() {
            return Ok(None);
        }

        Ok(Some(MissingTestGap {
            package_name: package.name.clone(),
            relative_source_file: relative_source,
            expected_test_file: ManifestResolver::relative_file(
                &package.manifest_dir,
                &expected_test,
            ),
        }))
    }

    fn should_ignore_package_source(
        &self,
        package: &PackageContext,
        relative_source: &str,
    ) -> bool {
        package.name.ends_with("-validation") && relative_source.starts_with("src/")
    }

    fn should_ignore_source_file(&self, relative_source: &str, source: &str) -> Result<bool> {
        if Self::is_entry_point(relative_source) {
            return Ok(true);
        }

        if relative_source.ends_with("/mod.rs") {
            return self.definition_analyzer.mod_file_is_import_only(source);
        }

        Ok(false)
    }

    // `src/bin/` is where cargo looks for a package's extra binaries, so a file
    // directly under it is an entry point in the same sense `src/main.rs` is,
    // and a mirrored test for its `fn main` would assert nothing. A file deeper
    // than that is a module belonging to one, and stays in scope.
    fn is_entry_point(relative_source: &str) -> bool {
        matches!(relative_source, "src/lib.rs" | "src/main.rs" | "build.rs")
            || relative_source
                .strip_prefix("src/bin/")
                .is_some_and(|rest| !rest.contains('/'))
    }
}
