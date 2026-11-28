# Implemented Features

This document describes the feature set currently shipped by
`cargo-twin4rust`. For the policy these implement, see
[RULES.md](RULES.md); for released versions, see
[CHANGELOG.md](../CHANGELOG.md).

## Version 0.8.0

### Analysis

- Mirrored-path check: `src/<path>/<name>.rs` expects
  `tests/<path>/<name>_tests.rs`, at any nesting depth
- Existence is the whole check — the expected test file is never opened
- Exclusion of entry points: `src/lib.rs`, `src/main.rs`, `build.rs`, and any
  file directly under `src/bin/`; a module nested inside a `src/bin/<name>/`
  binary stays in scope
- Exclusion of `mod.rs`, both via an import-only fast path and structurally,
  because no expected path is derived for that filename
- Exclusion of definition-only files: `struct`, `enum`, `union`, `type`,
  `trait`, `const` and `static` declarations alongside ignorable `use`,
  `extern crate` and bodiless `mod` items; a top-level macro or verbatim item
  keeps the file in scope
- Exclusion of trait impls carrying no methods, such as `impl Marker for T {}`
  and blanket alias impls
- Exclusion of data holders: a single `struct`, declared before its impls,
  whose inherent methods, across any number of inherent `impl` blocks, are all
  accessors in stern4rust's shapes — a getter reading one field, a plain
  constructor building its type from bare parameters, a `with_` step replacing
  one field — with at least one. A trait impl carrying a method, a second type
  or a top-level macro invocation forfeits the exclusion; only a method-free
  trait impl is tolerated beside the accessors
- Exclusion of humble adapters: a single type whose inherent methods are all
  either a constructor or a forwarding method. A constructor is a plain
  constructor of the data-holder rule's shape, or a trivial `new` whose fields
  may be any expression; a forwarding method returns nothing, has one
  statement, and that statement is a call. At least one must forward, so an
  empty `impl` does not qualify
- Exclusion of the `src/` tree of any package whose name ends in `-validation`,
  applied both at source-root resolution and per file
- `#[cfg(test)]` items stripped before any classification runs
- Files with no top-level items at all are reported rather than excluded
- A mirror for a source that earns none — a definition-only file or a data
  holder, carrying no trait default method — is reported as unearned and fails
  the run; a humble adapter, and definitions carrying a trait's default method,
  may have a mirror without one being demanded

### Resolution

- Package selection through `cargo metadata --no-deps`
- Repeatable `--package`; a single-root manifest needs none, a multi-package
  workspace requires at least one
- Source roots derived from production targets only — `test`, `bench`,
  `example` and `custom-build` kinds excluded; `lib`, `bin`, `proc-macro`,
  `rlib`, `dylib`, `cdylib` and `staticlib` included
- Fallback to `<manifest_dir>/src` for a package whose targets yield no root
- Source-root deduplication via an ordered set

### Reporting

- Human-readable report listing `package: source -> expected test`, and,
  under "Mirrored test files for sources that earn none", each unearned mirror
  as `package: source -> mirror`
- Gaps and unearned mirrors sorted by package name, then source path, then test
  path, so output is stable across runs and diffable in CI
- `summary: packages_with_gaps=N missing_files=M` trailer, with
  `unearned_mirrors=K` when there is one
- Exit code `1` when any gap or unearned mirror is found, `0` otherwise
- Path separators normalized to `/` so Windows and Linux output is identical; a
  `.rs` file whose path is not UTF-8 fails the run, named

### Packaging

- Two crates of one version: the command, published as `cargo-twin4rust` so
  cargo resolves `cargo twin4rust` (library `twin4rust_cli`), and the analysis,
  published as `twin4rust-core` (library `twin4rust`): `#![no_std]`, depending
  on quote and syn alone
- Conditional strip of the subcommand name cargo inserts at `argv[1]`, leaving
  direct invocation and a package named `twin4rust` both intact
- `--version` and `bin_name`-corrected help output

### Project

- `docs/RULES.md`, `docs/ARCHITECTURE.md`, `docs/ADRs/`, `CLAUDE.md`
- `just stage1` (lock sync, fmt, clippy, tests) and `just stage2`
  (`cargo xtask stage2`: technology-free core, house rules, unused
  dependencies, duplication, CRAP gate, mirrored-test self-analysis, file risk,
  dependency advisories, licences and sources), both running the same on
  Windows, Linux and macOS and both run by CI
- Test files named to mirror their subject, so the tool passes its own gate
