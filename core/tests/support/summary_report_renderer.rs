// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use twin4rust::analysis_report::AnalysisReport;
use twin4rust::traits::report_renderer::ReportRenderer;

// A renderer that writes nothing a reader would want: one `<package>=<gaps>`
// line per report it was handed, in the order it was handed them.
pub struct SummaryReportRenderer;

impl ReportRenderer for SummaryReportRenderer {
    fn render(&self, reports: &[AnalysisReport]) -> String {
        reports
            .iter()
            .map(|report| format!("{}={}\n", report.package_name(), report.missing_count()))
            .collect()
    }
}
