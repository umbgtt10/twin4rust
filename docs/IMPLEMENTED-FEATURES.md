# Implemented Features

This document describes the feature set currently shipped by
`cargo-twin4rust`. For the policy these implement, see
[RULES.md](RULES.md); for released versions, see
[CHANGELOG.md](../CHANGELOG.md).

## Version 0.4.0

### Analysis

- Mirrored-path check: `src/<path>/<name>.rs` expects
  `tests/<path>/<name>_tests.rs`, at any nesting depth
- Existence is the whole check — the expected test file is never opened
- Exclusion of entry points: `src/lib.rs`, `src/main.rs`, `build.rs`, and any
  file directly under `src/bin/`; a module nested inside a `src/bin/<name>/`
  binary stays in scope
- Exclusion of `mod.rs`, both via an import-only fast path and structurally,
  because no expected path is derived for that filename
- Exclusion of definition-only files: `struct`, `enum`, `type`, `trait`,
  `const` and `static` declarations alongside ignorable `use`, `extern crate`
  and bodiless `mod` items; a top-level macro or verbatim item keeps the file
  in scope
- Exclusion of trait impls carrying no methods, such as `impl Marker for T {}`
  and blanket alias impls
- Exclusion of single-type files whose only inherent `impl` is a trivial `new`:
  no generics, returns `Self`, single struct-literal body, no branching, no
  loops, no helper calls. Any second `impl` block forfeits the exclusion — a
  trait impl carrying a method or a second inherent impl both disqualify — and
  so does a top-level macro invocation. Only a method-free trait impl is
  tolerated beside the constructor
- Exclusion of humble adapters: a single type whose inherent methods are all
  either that trivial `new` or a forwarding method — returns nothing, one
  statement, and that statement is a call. At least one must forward, so an
  empty `impl` does not qualify
- Exclusion of the `src/` tree of any package whose name ends in `-validation`,
  applied both at source-root resolution and per file
- `#[cfg(test)]` items stripped before any classification runs
- Files with no top-level items at all are reported rather than excluded

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

- Human-readable report listing `package: source -> expected test`
- Gaps sorted by package name, then source path, then expected test path, so
  output is stable across runs and diffable in CI
- `summary: packages_with_gaps=N missing_files=M` trailer
- Exit code `1` when any gap is found, `0` otherwise
- Path separators normalized to `/` so Windows and Linux output is identical

### Packaging

- Published as `cargo-twin4rust` so cargo resolves `cargo twin4rust`; library
  name is `twin4rust`
- Conditional strip of the subcommand name cargo inserts at `argv[1]`, leaving
  direct invocation and a package named `twin4rust` both intact
- `--version` and `bin_name`-corrected help output

### Project

- `docs/RULES.md`, `docs/ARCHITECTURE.md`, `docs/ADRs/`, `CLAUDE.md`
- `just stage1` (fmt, clippy, tests) and `just stage2` (`cargo xtask stage2`:
  house rules, CRAP gate, mirrored-test self-analysis, file risk), both running
  the same on Windows, Linux and macOS and both run by CI
- Test files named to mirror their subject, so the tool passes its own gate
