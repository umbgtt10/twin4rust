// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::process::exit;

use anyhow::Result;

use crate::analyzer::Analyzer;
use crate::args::Args;
use crate::config::Config;
use crate::manifest_resolver::ManifestResolver;
use crate::report_printer::ReportPrinter;
use std::path::Path;

pub struct Runner;

impl Runner {
    pub fn run(args: Args) -> Result<()> {
        let config = Config::new(
            args.manifest_path().map(Path::to_path_buf),
            args.packages().to_vec(),
        );

        let resolver = ManifestResolver::new(config);
        let packages = resolver.resolve()?;
        let analyzer = Analyzer::new();
        let reports = analyzer.analyze_packages(&packages)?;

        let printer = ReportPrinter::new();
        printer.print(&reports);

        if reports.iter().any(|report| !report.is_empty()) {
            exit(1);
        }

        Ok(())
    }
}
