# twin4rust

## Meaning

`twin4rust` is a cargo subcommand that reports Rust source files with no
mirrored test file beside them — `src/<path>/<name>.rs` expects
`tests/<path>/<name>_tests.rs`.

It is not a coverage tool. It measures no lines and no branches, and it has no
opinion on whether existing tests are good. It answers one structural question:
does the mirrored test file exist?

`docs/RULES.md` is the canonical policy description. If the mirrored-test
policy changes, update `docs/RULES.md`, the tool logic and the integration
tests together. `docs/ARCHITECTURE.md` maps the code; `docs/ADRs/` holds the
load-bearing decisions.

It is self-contained.

## Boundary Rule

This repository is **SELF-CONTAINED**.

The LLM **SHALL NOT cross its boundaries without asking**.

That means:
- do not inspect, edit, or rely on files outside `twin4rust/` unless the user explicitly asks
- do not pull assumptions from sibling repositories or crates
- do not propose cross-repository changes by default

## Quality Gates

### Mandatory after every change to any member's `src/` or `tests/`

Run gates:

`just stage1`
`just stage2`

If either gate is not green, the work is not complete.

Both run identically on Windows, Linux and macOS, and CI runs the same two
commands -- there is no second definition of the gates to drift out of step.

Stage 1 is formatting, clippy and tests -- cargo built-ins only, so it works on
a fresh checkout with none of the house tools installed.

Stage 2 is `cargo xtask stage2` -- a real crate under `xtask/`, gated like any
other code, rather than a script. Each gate is a `Gate` implementation
constructed against a `CommandRunner` trait, so the argument lists and the
failure messages are covered by `xtask`'s own integration tests. It runs eight
gates, in this order:

| gate | asks |
|---|---|
| Technology-free core | does `core` still touch no technology |
| `cargo stern4rust` | do the house coding rules hold |
| `cargo shear` | is every declared dependency used, in the section that uses it |
| `cargo dry4rust` | did this change add duplicated code |
| `cargo crap4rust` | is any function complex and untested |
| `cargo twin4rust` | does every source file have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk too high |
| `cargo deny` | do the dependencies carry an advisory, a refused licence or a foreign source |

The technology-free gate runs **first**. It is the xtask's own -- no tool to
install -- and asks three things of `core`: that its manifest names only
`quote` and `syn`, in any section (read from `cargo metadata --no-deps`; a
package missing from the metadata fails); that `core/src/lib.rs` carries
`#![no_std]` and `#![forbid(unsafe_code)]`, which hand `core/src` to the
compiler; and that no file under `core/src` or `core/tests` reaches
`std::{env,fs,io,net,os,process}`, a printing macro, a file-system method of
`Path` or `extern crate std`, read with syn so a string literal is data. The
`core/src` scan is a backstop to the compiler; `core/tests`, where std is
linked, depends on it. Calls inside a macro's arguments are not seen, and a
method of one's own named like a file-system one would be flagged -- which is
why the port asking whether a mirror exists calls it `contains`. Its fix is
moving code into `cli`, which changes what every gate behind it measures, and
it takes a second.

stern4rust runs **second** because its corrections are renames, file moves and
directory splits: a layout it is about to reject is a layout the others would
have measured for nothing. Its findings are also the cheapest to act on.

cargo-shear runs **third**: its corrections edit manifests only -- a
dependency removed, or moved to the section that uses it -- so they change
nothing the gates behind it measure, and it takes a second. It runs `--locked`,
reading the lock file and never writing it, and `--deny-warnings`, so an
optional dependency no feature's code uses fails as an unused one does. A
dependency it cannot see used -- linked only for what it provides, declared
only to switch on a feature, used only by generated code -- goes in
`[package.metadata.cargo-shear] ignored`, and the gate refuses an entry without
its reason in the same table's `reasons`; there are none today. Pinned at
1.14.0, as CI installs it.

`dry4rust` runs **fourth** for the same reason stern4rust runs early: removing
a duplicate moves code between files, which changes what the three behind it
measure. It makes one run over the workspace with the trees it must not judge
excluded by full path -- `xtask`, which is not published, and every `tests/`,
which repeat their arrangement by design -- so it judges `core/src` and
`cli/src` together and sees a copy from one into the other. It checks with zero
ceilings against `dry4rust-baseline.json`, the duplication already there when
the gate arrived. So it fails on what a change adds, not on what it
inherited; a duplicate removed is admitted, a copy added to a recorded group is
not. The baseline is empty: at the gate's floor `core/src` had no duplication to
inherit, so today any group at all fails the gate.

It counts only code units of 25 AST nodes or more, the floor the rest of the
family uses. Below it sit units that look alike because there is one way to
write them: three `new` constructors whose whole body is a one-field struct
literal, and `AnalysisReport`'s two one-line accessors. At dry4rust's default of 10 those are
two groups that sharing would not improve; at 25 they drop out, and a copied
method body of any real size is still caught.

Re-record the baseline only to drop groups that are gone, never to admit new
ones, and at the same floor -- a baseline matches only at the floor it was
recorded at: `cargo dry4rust --path "$PWD" --exclude "$PWD/xtask" --exclude "$PWD/core/tests" --exclude "$PWD/cli/tests" --min-nodes 25 --baseline "$PWD/dry4rust-baseline.json" baseline`.

crap4rust, the self-analysis and iceberg4rust measure the two published crates,
`twin4rust-core` and `cargo-twin4rust`: their bill is the tool's, not the
xtask's, which is held to the house rules alone.

All twenty-two of its rules are enforced, with nothing skipped, nothing
unconfigured and no baseline: `stern4rust.toml` carries no `skip` and no `rules`
selection. That includes `returned-mutations`, which is the "no `&mut` input
parameters" standard below made mechanical. `docs/header.txt` holds the
three-line header every `.rs` file carries and `stern4rust.toml` names it -- in
the config rather than the gate script, so a hand-run of `cargo stern4rust`
checks exactly what the gate checks. `spdx-matches-manifest` reads the
`license` the workspace manifest declares.

The stern gate is scoped to `twin4rust-core`, `cargo-twin4rust` **and**
`xtask`. The crate that runs the gates is not exempt from them.

cargo-deny runs **last**, and is the one gate that can turn red with nothing
changed: its advisories come from the RustSec database, so a newly published
one fails an untouched checkout, and it fetches that database over the network.
That is the point of it, and running last keeps it from hiding what the house
gates say about a change. `deny.toml` is the same file in every *4rust
repository and slotgate: licences MIT, Apache-2.0 and Unicode-3.0 -- the union
of what their trees carry -- crates.io as the only source, duplicate versions a
warning, and a wildcard requirement refused except a path dependency in an
unpublished crate. An accepted advisory goes in `[advisories] ignore` with its
id and its reason. The gate fetches the advisory database first, on its own and
up to three times, then checks `--frozen` -- locked and offline -- so a fetch
that keeps failing reports the gate as unable to run, never as an advisory
found. It decodes cargo-deny's exit code, one bit per failed check, into the
checks' names; a `deny.toml` it cannot read exits 1, as advisories does, and the
streamed output says which.

`cargo install just`
`cargo install cargo-llvm-cov`
`cargo install cargo-stern4rust`
`cargo install cargo-dry4rust`
`cargo install cargo-crap4rust`
`cargo install cargo-iceberg4rust`
`cargo install --locked cargo-shear --version 1.14.0`
`cargo install --locked cargo-deny --version 0.20.2`

cargo-twin4rust is deliberately not in that list. The self-analysis gate builds
it from this checkout -- `cli`'s binary, pointed at `core` and `cli` -- so the
tree being checked is the tree being changed.

## Layout

A workspace of three members, with a virtual root that holds no package:

| member | what it is |
|---|---|
| `core/` | the analysis: package `twin4rust-core`, library `twin4rust`. `#![no_std]`, `#![forbid(unsafe_code)]`, no technology -- see "Technology-free core" below. Published. |
| `cli/` | the command: package `cargo-twin4rust`, library `twin4rust_cli`, binary `cargo-twin4rust`. `Args`, the composition root `EntryPoint`, and every adapter. Published. |
| `xtask/` | gate orchestration, run as `cargo xtask`. `publish = false`. |

`core` and `cli` share the workspace's `version`. `[workspace.lints.rust]`
forbids `unsafe_code` in every member and every target; `core` says so at its
crate root as well.

Giving each crate a directory of its own is load-bearing rather than
tidy-minded. While the crate sat at the repository root, its package directory
*was* the repository root, so `cargo stern4rust` walked `xtask/tests/**` and
reported 78 offences for test functions "in the source tree" -- files belonging
to a different package entirely. Scoping with `--package` does not help, because
the scope is the directory. Giving each crate its own directory is what
separates them.

The twin4rust gate runs this tool against itself. A tool that enforces a rule it
does not satisfy is not worth installing, so every behaviour-bearing source file
here keeps its own mirrored `<name>_tests.rs`.

## Technology-free core

`core` touches no technology -- no cargo, file system, process, environment,
argument parsing or stdout -- so mutation testing it needs nothing but its own
tests, which run in a few hundredths of a second. `.cargo/mutants.toml` scopes
cargo-mutants to `core/src`, each mutant tested by core's tests only.

Whatever technology core needs is a port: a trait in `core/src/traits/`
(`WorkspaceReader`, `SourceTree`, `ReportRenderer`), `Sync` so a `Runner`
holding it stays `Send`; the data it speaks in `core/src/workspace/` or beside
its user; an in-memory fake for core's tests in `core/tests/support/`; and the
real adapter in `cli/src/system/`, composed in `cli`'s `EntryPoint`. Paths in
core are `SourcePath`s, text whose only separator is `/`; `cli`'s `PortablePath`
converts at the edge, and a `.rs` path that is not UTF-8 fails the run rather
than being reported under a lossy name. core's failures are typed (`RunFailure`
and the records it wraps) and keep their causes, so `cli`'s `main` prints the
same chain anyhow always printed.

## Publishing

`cli` publishes as `cargo-twin4rust` so cargo resolves `cargo twin4rust`,
matching `cargo-crap4rust` and `cargo-dry4rust`. The library `twin4rust`
publishes as `twin4rust-core`, with a short README of its own.

A release publishes **both**, `twin4rust-core` first, since `cli` names it by
version: `cargo publish --workspace` orders them and leaves `xtask` out. Their
shared version moves once, in `[workspace.package]` and in the root's
`twin4rust-core` dependency together.

`cli`'s `EntryPoint` strips the repeated subcommand name that cargo inserts as
`argv[1]`; running the binary directly does not repeat it, so the strip is
conditional. `Args::without_cargo_subcommand` owns that rule and is tested.

Before publishing, `cargo publish --workspace --dry-run` must succeed.

## Orthogonality, trait surface and cognitive complexity

**When changing productive code, always maximize orthogonality and testable surface through traits, and minimize cognitive complexity.**

Specifically:
- prefer extracting behavior behind traits so individual pieces can be tested and swapped independently
- prefer small, focused methods with a single responsibility over large methods with many branches
- prefer named structs with methods over free functions operating on external state
- when `crap4rust` or a reviewer flags a function as too complex, reduce it by extracting internal structs with methods and adding integration coverage — not by extracting standalone helper functions
- never increase cognitive complexity to pass a test; find the root cause and fix it there
- make constructors depend on traits, not directly on concrete implementations
- ALL dependencies are injected through the SINGLE constructor and stored in the struct
- apply the same split recursively to nested dependencies: trait first, state/data model second, concrete implementation third

## User coding standards

- one object per file; records where record-home puts them
- no unnecessary comments in code
- unit tests are not allowed. Only integration tests are
- consolidate scattered functions inside structs as appropriate
- no `&mut` input parameters; prefer return values
- only use `pub mod` in `mod.rs` and `lib.rs`
- split test files so there is one test file per source file, named `<source file name>_tests.rs`
- in `all_tests.rs`, reference test files one by one without `#[path = ...]`
- apply AAA (`Arrange`, `Act`, `Assert`) structure to tests with blank-line separation between the three sections
- use `// Arrange & Act` if there is no separate `Arrange`
- use `// Act & Assert` if there is no separate `Act`
- add the repository copyright and license header to every Rust source file
- tests should be named as follows `<method under test>_<test description>_<result>`
- do not use fully qualified paths; use `use` imports instead
