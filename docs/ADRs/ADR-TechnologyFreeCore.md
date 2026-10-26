# ADR-TechnologyFreeCore

- **Status:** Accepted
- **Date:** 2026-10-26

## Context

The tool is meant to be mutation-tested, and mutation testing pays for every
mutant with a build and a test run. While one crate held everything, a mutant
anywhere rebuilt a crate that linked cargo_metadata, clap and walkdir, and its
tests ran `cargo metadata` four times and wrote thirty-three temporary
directories. `Runner::run` ended the process with `exit(1)` when it found a gap,
so a mutant there could take the whole test binary down with it. The analysis --
which file needs a mirror and where -- is what deserves mutating, and it was
mixed with code whose only job is to talk to the machine.

## Decision

The analysis lives in a technology-free crate, `twin4rust-core`: `#![no_std]`
and `#![forbid(unsafe_code)]`, depending on `quote` and `syn` only, declaring a
trait for every piece of technology it needs, and returning its report as a
string and its verdict as a flag; a separate crate, `cargo-twin4rust`,
implements those traits with the real technology, parses the arguments, prints,
and decides the exit code.

## Forcing constraints / Evidence

`cargo mutants --list` names 244 mutants, all in the core; the workspace held
305 before the split, 108 of them in `xtask`. The core's 215 tests touch no disk
and start no process, and run in under a tenth of a second where the 187 before
took about a third. Measured on the same machine, one expression changed in
`TestFileResolver::expected_test_file` as cargo-mutants changes it, medians of
three with other jobs holding its load average at six to eight: building the
crate went from 0.49 s to 0.26 s, building and running its tests from 0.95 s to
0.49 s, and a clean build of its tests -- what each cargo-mutants worker pays
first -- from 19.4 s to 6.8 s.

The binary's behaviour did not move: stdout, stderr and exit codes were compared
byte for byte against 0.5.1 on fourteen invocations after every step.

## Rejected alternatives

- **One crate, with cargo-mutants' `exclude_globs` on the technology.** Scopes
  the mutants but not the build or the tests, and nothing would stop the next
  change reaching for `std::fs` inside the analysis.
- **An output port for stdout.** The core returns a string; printing it is the
  composition root's one line.
- **Leaving the text report to `cli`.** It is pure string building, and the
  order and summary it prints are worth mutating; it is the core's
  implementation of the `ReportRenderer` port.
- **`clippy.toml`'s `disallowed-*` lists.** Type-resolved, but blind to
  manifests, and with `#![no_std]` the compiler already guards the source.

## Consequences

Two published crates, `twin4rust-core` first, sharing one version. A new
technology need is a trait in `core/src/traits/`, an in-memory fake in
`core/tests/support/` and an adapter in `cli/src/system/`, composed in
`EntryPoint`; the traits are `Sync`, so a `Runner` holding them stays `Send`.
Paths in the core are `SourcePath`s with `/` as their only separator, converted
by `cli` at the edge; a `.rs` file whose path is not UTF-8 fails the run instead
of being reported under a name it does not have. The core's errors are typed and
keep their causes, so `cli`'s `main` prints the chain it always printed.

## Enforcement

The "Technology-free core" gate, first in stage 2: core's manifest names only
`quote` and `syn` in any section; `core/src/lib.rs` carries `#![no_std]` and
`#![forbid(unsafe_code)]`; no file under `core/src` or `core/tests` reaches
`std::{env,fs,io,net,os,process}`, a printing macro, a file-system method of
`Path` or `extern crate std`. `.cargo/mutants.toml` scopes cargo-mutants to
`core/src`, each mutant tested by core's tests only.

## Related

- [ADR-CargoSubcommandPackaging](ADR-CargoSubcommandPackaging.md)
- [ADR-HumbleAdaptersAtUntestableBoundaries](ADR-HumbleAdaptersAtUntestableBoundaries.md)
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- [`../history/[IMPLEMENTED] CORE_CLI_SPLIT.md`](../history/%5BIMPLEMENTED%5D%20CORE_CLI_SPLIT.md)
