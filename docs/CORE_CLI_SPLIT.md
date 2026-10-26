# A technology-free core and a cli crate

Status: **in progress** -- one commit per slice, in the order below.

## Why

Mutation testing pays for every mutant with a build and a test run. A mutant in
code that runs `cargo metadata`, walks a disk or prints is slow to kill, and a
test that needs a workspace to run in is slow to run: today four of core's
tests spawn `cargo metadata`, thirty-three write to disk, and `Runner::run`
ends the process with `exit(1)` when it finds a gap. So package selection, the
mirrored-path rule, its exemptions and the text report move into a crate that
touches no technology at all, and mutation testing is scoped to that crate
alone.

## The shape

The workspace grows a third member:

| member | package | library | what it holds |
|---|---|---|---|
| `core/` | `twin4rust-core` | `twin4rust` | `#![no_std]`, `#![forbid(unsafe_code)]`; package selection, the source roots, the mirrored-path rule, the exemptions and the text report. Depends on `quote` and `syn` only |
| `cli/` | `cargo-twin4rust` | `twin4rust_cli` | the binary, `Args` (clap), the composition root `EntryPoint`, and every adapter: cargo metadata and the file system |
| `xtask/` | `xtask` | `xtask` | the gates |

`cli` depends on `core`, never the reverse. Both published crates share one
workspace version.

### Ports

Core declares a trait for everything technological it needs, implemented in
`cli/src/system/` and, for core's own tests, by in-memory fakes under
`core/tests/support/`:

| port (`core/src/traits/`) | what it answers | cli adapter |
|---|---|---|
| `SourceTree` | the `.rs` files under a root, a file's text, and whether a path -- the mirrored test file -- exists | `WalkdirSourceTree` |
| `WorkspaceReader` | the workspace's packages and targets | `CargoWorkspaceReader` (`cargo metadata --no-deps`) |
| `ReportRenderer` | the reports as text | `TextReportRenderer`, which lives in core |

The ports are `Sync`, so a `Runner` holding them stays `Send`. Core works on
its own records (`workspace/`: `WorkspaceDescription`, `PackageDescription`,
`TargetDescription`), on `SourcePath` -- a path held as text with `/` as its
only separator, which cli converts to at the edge -- and fails with typed
errors (`SourceReadFailure`, `WorkspaceReadFailure`, `ParseFailure`,
`RunFailure`). It returns its output as a string; printing it and turning the
verdict into an exit code is `EntryPoint`'s.

The binary's behaviour does not change: stdout, stderr and exit codes are
compared byte for byte against 0.5.1 after every slice that touches them. One
exception is deliberate: a `.rs` file whose path is not UTF-8 fails the run
naming it, where 0.5.1 reported it under a name it does not have.

### The gate

A stage-2 gate, first in the run, keeps core technology-free: its manifest may
name only `quote` and `syn` in any section, its crate root must carry
`#![no_std]` and `#![forbid(unsafe_code)]`, and no file under `core/src` or
`core/tests` may reach `std::{env,fs,io,net,os,process}`, the printing macros,
the file-system methods or `extern crate std`.

### Mutation testing

`.cargo/mutants.toml` scopes cargo-mutants to `core/src`; each mutant runs
core's tests only.

## Slices

| # | slice | status |
|---|---|---|
| S0 | this record | done |
| S1 | `DryGate`: one run over the workspace, with the trees it must not judge excluded by full path | done |
| S2 | the split: `cli` takes `main.rs`, `Args` and a new `EntryPoint`; core is renamed; one workspace version; `[workspace.lints]` forbids unsafe code; `Runner::run` returns an exit code instead of calling `exit` | done |
| S3 | the `SourceTree` port and `SourcePath`; `Analyzer` reads through it; the walker becomes cli's `WalkdirSourceTree` | |
| S4 | the `WorkspaceReader` port and core's workspace records; `CargoWorkspaceReader` in cli | |
| S5 | the `ReportRenderer` port and `RunOutcome`: the text report in core, printing and the exit code in `EntryPoint` | |
| S6 | `#![no_std]`: alloc imports, typed errors; anyhow leaves core | |
| S7 | the technology-free gate, first in stage 2 | |
| S8 | `.cargo/mutants.toml` | |
| S9 | docs, and this record moves to `docs/history/` | |

## Tests that move

No test is deleted. Tests whose subject moves go with it:

| from | to |
|---|---|
| `core/tests/args_tests.rs` (9) | `cli/tests/invocation/args_tests.rs` |
| `runner_tests`: `run_against_a_manifest_that_does_not_exist_returns_an_error`, `run_against_this_package_finds_no_missing_mirror` | `cli/tests/invocation/entry_point_tests.rs`, same names, argv-driven |
| `core/tests/source_walker_tests.rs` (6, `walk_*`) | `cli/tests/system/walkdir_source_tree_tests.rs`, renamed `rust_files_*` |
| `manifest_resolver_tests::resolve_against_this_package_returns_it_with_its_source_root` | `cli/tests/system/cargo_workspace_reader_tests.rs`, same name |
| `core/tests/report_printer_tests.rs` (4) | `core/tests/text_report_renderer_tests.rs` |

The analyzer's tests keep their names and assertions and arrange an in-memory
source tree instead of a temporary directory; the resolver's and the
collector's arrange core's records instead of cargo's JSON.

The four printer tests are rewritten because the method they called no longer
exists: `print_empty_reports_does_not_panic`,
`print_reports_with_missing_gaps_does_not_panic`,
`print_reports_with_multiple_packages_and_gaps_does_not_panic` and
`print_reports_with_no_gaps_does_not_panic` become
`render_empty_reports_says_every_expectation_is_satisfied`,
`render_reports_with_missing_gaps_lists_each_and_the_summary`,
`render_reports_with_multiple_packages_and_gaps_sorts_them_and_counts_both` and
`render_reports_with_no_gaps_says_every_expectation_is_satisfied`, which assert
the exact text where the old ones asserted only that nothing panicked, on the
same reports. `run_against_this_package_finds_no_missing_mirror` asserts the
exit code is success, which `exit(1)` used to make implicit by killing the test
binary. `config_clone_produces_independent_copy` loses its `manifest_path` line
with the field, which moves into `CargoWorkspaceReader`.
