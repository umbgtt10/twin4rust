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

const GAPS_FOUND: u8 = 1;

pub struct Runner<'a> {
    analyzer: Analyzer<'a>,
}

impl<'a> Runner<'a> {
    pub fn new(source_tree: &'a dyn SourceTree) -> Self {
        Self {
            analyzer: Analyzer::new(source_tree),
        }
    }

    pub fn run(&self, config: &Config) -> Result<ExitCode> {
        let resolver = ManifestResolver::new(config.clone());
        let packages = resolver.resolve()?;
        let reports = self.analyzer.analyze_packages(&packages)?;

        let printer = ReportPrinter::new();
        printer.print(&reports);

        if reports.iter().any(|report| !report.is_empty()) {
            return Ok(ExitCode::from(GAPS_FOUND));
        }

        Ok(ExitCode::SUCCESS)
    }
}
