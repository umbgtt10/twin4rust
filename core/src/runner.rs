// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::process::ExitCode;

use anyhow::Result;

use crate::analyzer::Analyzer;
use crate::config::Config;
use crate::manifest_resolver::ManifestResolver;
use crate::report_printer::ReportPrinter;

const GAPS_FOUND: u8 = 1;

pub struct Runner;

impl Runner {
    pub fn run(config: &Config) -> Result<ExitCode> {
        let resolver = ManifestResolver::new(config.clone());
        let packages = resolver.resolve()?;
        let analyzer = Analyzer::new();
        let reports = analyzer.analyze_packages(&packages)?;

        let printer = ReportPrinter::new();
        printer.print(&reports);

        if reports.iter().any(|report| !report.is_empty()) {
            return Ok(ExitCode::from(GAPS_FOUND));
        }

        Ok(ExitCode::SUCCESS)
    }
}
