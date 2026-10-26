// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Result;
use std::env::args;
use std::process::ExitCode;
use twin4rust_cli::invocation::entry_point::EntryPoint;

// Reading the real process argv is the one thing no test can reach, so it is
// all this binary does. What it hands the argv to is public and tested.
fn main() -> Result<ExitCode> {
    EntryPoint::new().run(args())
}
