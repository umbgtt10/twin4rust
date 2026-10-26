// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use twin4rust::runner::Runner;

use crate::invocation::args::Args;
use crate::system::walkdir_source_tree::WalkdirSourceTree;

// The composition root: the command line in, the report out on stdout, an exit
// code back. The one place the real file system is chosen; everything it
// composes is the core's.
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
        let source_tree = WalkdirSourceTree::new();
        Runner::new(&source_tree).run(&args.config())
    }
}
