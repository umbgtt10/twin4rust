// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use crate::analysis_report::AnalysisReport;
use crate::missing_test_gap::MissingTestGap;
use crate::traits::report_renderer::ReportRenderer;

// The report for a reader: every gap in a stable order, then a summary line.
// Pure string building.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextReportRenderer;

impl TextReportRenderer {
    pub fn new() -> Self {
        Self
    }

    fn collect_missing<'a>(&self, reports: &'a [AnalysisReport]) -> Vec<&'a MissingTestGap> {
        reports
            .iter()
            .flat_map(|report| report.missing().iter())
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

impl ReportRenderer for TextReportRenderer {
    fn render(&self, reports: &[AnalysisReport]) -> String {
        let missing = self.collect_missing(reports);

        let mut out = String::from("twin4rust report\n\n");

        if missing.is_empty() {
            out.push_str("All mirrored test-gap expectations are satisfied.\n");
            return out;
        }

        out.push_str("Files without a matching mirrored test file:\n");
        for gap in self.sorted(&missing) {
            let _ = writeln!(
                out,
                "- {}: {} -> {}",
                gap.package_name, gap.relative_source_file, gap.expected_test_file
            );
        }

        let _ = write!(
            out,
            "\nsummary: packages_with_gaps={} missing_files={}\n",
            reports.iter().filter(|report| !report.is_empty()).count(),
            missing.len()
        );
        out
    }
}
