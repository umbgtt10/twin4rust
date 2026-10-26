// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::process::ExitCode;

use anyhow::Result;

use crate::analyzer::Analyzer;
use crate::config::Config;
use crate::manifest_resolver::ManifestResolver;
use crate::report_printer::ReportPrinter;
use crate::traits::source_tree::SourceTree;
use crate::traits::workspace_reader::WorkspaceReader;

const GAPS_FOUND: u8 = 1;

pub struct Runner<'a> {
    resolver: ManifestResolver<'a>,
    analyzer: Analyzer<'a>,
}

impl<'a> Runner<'a> {
    pub fn new(workspace_reader: &'a dyn WorkspaceReader, source_tree: &'a dyn SourceTree) -> Self {
        Self {
            resolver: ManifestResolver::new(workspace_reader),
            analyzer: Analyzer::new(source_tree),
        }
    }

    pub fn run(&self, config: &Config) -> Result<ExitCode> {
        let packages = self.resolver.resolve(config)?;
        let reports = self.analyzer.analyze_packages(&packages)?;

        let printer = ReportPrinter::new();
        printer.print(&reports);

        if reports.iter().any(|report| !report.is_empty()) {
            return Ok(ExitCode::from(GAPS_FOUND));
        }

        Ok(ExitCode::SUCCESS)
    }
}
