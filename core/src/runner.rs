// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::analyzer::Analyzer;
use crate::config::Config;
use crate::manifest_resolver::ManifestResolver;
use crate::run_failure::RunFailure;
use crate::run_outcome::RunOutcome;
use crate::traits::report_renderer::ReportRenderer;
use crate::traits::source_tree::SourceTree;
use crate::traits::workspace_reader::WorkspaceReader;

pub struct Runner<'a> {
    resolver: ManifestResolver<'a>,
    analyzer: Analyzer<'a>,
    renderer: &'a dyn ReportRenderer,
}

impl<'a> Runner<'a> {
    pub fn new(
        workspace_reader: &'a dyn WorkspaceReader,
        source_tree: &'a dyn SourceTree,
        renderer: &'a dyn ReportRenderer,
    ) -> Self {
        Self {
            resolver: ManifestResolver::new(workspace_reader),
            analyzer: Analyzer::new(source_tree),
            renderer,
        }
    }

    // Printing the output, and turning the verdict into an exit code, is the
    // caller's.
    pub fn run(&self, config: &Config) -> Result<RunOutcome, RunFailure> {
        let packages = self.resolver.resolve(config)?;
        let reports = self.analyzer.analyze_packages(&packages)?;

        Ok(RunOutcome {
            output: self.renderer.render(&reports),
            has_gaps: reports.iter().any(|report| !report.is_empty()),
        })
    }
}
