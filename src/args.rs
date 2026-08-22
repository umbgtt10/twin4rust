// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "cargo-twin4rust")]
#[command(bin_name = "cargo twin4rust")]
#[command(version)]
#[command(about = "Check mirrored test coverage expectations for Rust packages")]
pub struct Args {
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,

    #[arg(long = "package")]
    pub packages: Vec<String>,
}

impl Args {
    /// Cargo invokes `cargo twin4rust` as `cargo-twin4rust twin4rust ...`, so the
    /// subcommand name arrives as an extra leading argument that clap would
    /// otherwise reject. Running the binary directly does not repeat it.
    pub fn without_cargo_subcommand<I>(args: I) -> Vec<String>
    where
        I: IntoIterator<Item = String>,
    {
        let args: Vec<String> = args.into_iter().collect();
        if args.get(1).map(String::as_str) != Some("twin4rust") {
            return args;
        }
        let mut forwarded = Vec::with_capacity(args.len() - 1);
        forwarded.extend(args.iter().take(1).cloned());
        forwarded.extend(args.into_iter().skip(2));
        forwarded
    }
}
