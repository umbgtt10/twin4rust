// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::process::command_runner::CommandRunner;

pub struct TwinSelfGate<'a> {
    runner: &'a dyn CommandRunner,
    binary: String,
    manifest_path: String,
    packages: Vec<String>,
}

impl<'a> TwinSelfGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        binary: String,
        manifest_path: String,
        packages: Vec<String>,
    ) -> Self {
        Self {
            runner,
            binary,
            manifest_path,
            packages,
        }
    }
}

impl Gate for TwinSelfGate<'_> {
    fn label(&self) -> String {
        String::from("twin4rust self-analysis")
    }

    fn run(&self) -> Result<(), String> {
        // Built from this checkout rather than taken from an install: a tool
        // that checks its own mirrored tests has to check the tree being
        // changed.
        //
        // Two different flags that look alike sit on either side of the `--`.
        // Before it, `--bin` tells cargo which binary to build. After it, the
        // tool gets its own `--package`, which the workspace root now requires:
        // a virtual manifest names no single package, so the tool cannot guess
        // which one to analyse and says so rather than picking.
        let mut args = vec![
            String::from("run"),
            String::from("--quiet"),
            String::from("--bin"),
            self.binary.clone(),
            String::from("--"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }

        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            _ => Err(String::from("source files without a mirrored test")),
        }
    }
}
