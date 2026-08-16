// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Result;
use twin4rust::args::Args;
use twin4rust::runner::Runner;

fn main() -> Result<()> {
    let args = Args::parse_args();
    Runner::run(args)
}
