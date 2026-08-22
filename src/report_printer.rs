// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::analysis_report::AnalysisReport;
use crate::missing_test_gap::MissingTestGap;

#[derive(Default)]
pub struct ReportPrinter;

impl ReportPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&self, reports: &[AnalysisReport]) {
        let missing = self.collect_missing(reports);

        println!("twin4rust report");
        println!();

        if missing.is_empty() {
            println!("All mirrored test-gap expectations are satisfied.");
            return;
        }

        println!("Files without a matching mirrored test file:");
        for gap in self.sorted(&missing) {
            println!(
                "- {}: {} -> {}",
                gap.package_name, gap.relative_source_file, gap.expected_test_file
            );
        }

        println!();
        println!(
            "summary: packages_with_gaps={} missing_files={}",
            reports.iter().filter(|report| !report.is_empty()).count(),
            missing.len()
        );
    }

    fn collect_missing<'a>(&self, reports: &'a [AnalysisReport]) -> Vec<&'a MissingTestGap> {
        reports
            .iter()
            .flat_map(|report| report.missing.iter())
            .collect()
    }

    fn sorted<'a>(&self, missing: &[&'a MissingTestGap]) -> Vec<&'a MissingTestGap> {
        let mut sorted = missing.to_vec();
        sorted.sort_by(|left, right| {
            left.package_name
                .cmp(&right.package_name)
                .then_with(|| left.relative_source_file.cmp(&right.relative_source_file))
                .then_with(|| left.expected_test_file.cmp(&right.expected_test_file))
        });
        sorted
    }
}
