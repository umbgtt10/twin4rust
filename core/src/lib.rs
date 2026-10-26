// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod analysis_report;
pub mod analyzer;
pub mod behaviourless_impl_detector;
pub mod config;
pub mod definition_analyzer;
pub mod humble_adapter_detector;
pub mod manifest_resolver;
pub mod missing_test_gap;
pub mod package_context;
pub mod parse_failure;
pub mod run_failure;
pub mod run_outcome;
pub mod runner;
pub mod target_root_collector;
pub mod test_file_resolver;
pub mod text_report_renderer;
pub mod traits;
pub mod trivial_constructor_detector;
pub mod workspace;
