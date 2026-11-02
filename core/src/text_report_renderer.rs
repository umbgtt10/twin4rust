// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use crate::analysis_report::AnalysisReport;
use crate::missing_test_gap::MissingTestGap;
use crate::traits::report_renderer::ReportRenderer;
use crate::unearned_mirror::UnearnedMirror;

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

    fn collect_unearned<'a>(&self, reports: &'a [AnalysisReport]) -> Vec<&'a UnearnedMirror> {
        let mut unearned: Vec<&UnearnedMirror> = reports
            .iter()
            .flat_map(|report| report.unearned().iter())
            .collect();
        unearned.sort_by(|left, right| {
            left.package_name
                .cmp(&right.package_name)
                .then_with(|| left.relative_source_file.cmp(&right.relative_source_file))
                .then_with(|| left.mirror_test_file.cmp(&right.mirror_test_file))
        });
        unearned
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
        let unearned = self.collect_unearned(reports);

        let mut out = String::from("twin4rust report\n\n");

        if missing.is_empty() && unearned.is_empty() {
            out.push_str("All mirrored test-gap expectations are satisfied.\n");
            return out;
        }

        if !missing.is_empty() {
            out.push_str("Files without a matching mirrored test file:\n");
            for gap in self.sorted(&missing) {
                let _ = writeln!(
                    out,
                    "- {}: {} -> {}",
                    gap.package_name, gap.relative_source_file, gap.expected_test_file
                );
            }
        }

        if !unearned.is_empty() {
            if !missing.is_empty() {
                out.push('\n');
            }
            out.push_str("Mirrored test files for sources that earn none:\n");
            for mirror in &unearned {
                let _ = writeln!(
                    out,
                    "- {}: {} -> {}",
                    mirror.package_name, mirror.relative_source_file, mirror.mirror_test_file
                );
            }
        }

        let _ = write!(
            out,
            "\nsummary: packages_with_gaps={} missing_files={}",
            reports.iter().filter(|report| !report.is_empty()).count(),
            missing.len()
        );
        if !unearned.is_empty() {
            let _ = write!(out, " unearned_mirrors={}", unearned.len());
        }
        out.push('\n');
        out
    }
}
