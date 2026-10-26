// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use twin4rust::runner::Runner;
use twin4rust::text_report_renderer::TextReportRenderer;

use crate::invocation::args::Args;
use crate::system::cargo_workspace_reader::CargoWorkspaceReader;
use crate::system::walkdir_source_tree::WalkdirSourceTree;

const GAPS_FOUND: u8 = 1;

// The composition root: the command line in, the report out on stdout, an exit
// code back. The one place cargo metadata, the real file system and stdout are
// chosen; everything it composes is the core's, tested without them.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntryPoint;

impl EntryPoint {
    pub fn new() -> Self {
        Self
    }

    pub fn run<I>(&self, argv: I) -> Result<ExitCode>
    where
        I: IntoIterator<Item = String>,
    {
        let args = Args::parse_from(Args::without_cargo_subcommand(argv));
        let workspace_reader =
            CargoWorkspaceReader::new(args.manifest_path().map(Path::to_path_buf));
        let source_tree = WalkdirSourceTree::new();
        let renderer = TextReportRenderer::new();
        let outcome =
            Runner::new(&workspace_reader, &source_tree, &renderer).run(&args.config())?;

        print!("{}", outcome.output);

        // 1 for a gap, and 1 for an error via the `Err` path: a gate reading
        // twin4rust asks only whether every mirror exists.
        Ok(if outcome.has_gaps {
            ExitCode::from(GAPS_FOUND)
        } else {
            ExitCode::SUCCESS
        })
    }
}
