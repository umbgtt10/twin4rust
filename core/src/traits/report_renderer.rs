// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;

use crate::analysis_report::AnalysisReport;

pub trait ReportRenderer: Sync {
    fn render(&self, reports: &[AnalysisReport]) -> String;
}
