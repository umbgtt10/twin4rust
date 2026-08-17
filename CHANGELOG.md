# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-08-17

Two defects in how binaries are handled, both found by pointing the tool at the
first crate to declare a `[[bin]]` under `src/bin/`. A patch rather than a
minor: neither changes policy, and each only stops the tool reporting something
it should never have reported.

### Fixed

- A binary under `src/bin/` is no longer reported as missing a mirrored test.
  Entry points were excluded by matching three literal paths — `src/lib.rs`,
  `src/main.rs`, `build.rs` — so `src/bin/board_ctl.rs` fell through and was
  asked for `tests/bin/board_ctl_tests.rs`, a mirrored test for a `fn main`.
  `src/bin/` is where cargo looks for a package's extra binaries, so a file
  directly under it is an entry point in the same sense `src/main.rs` is. Depth
  is what distinguishes them: `src/bin/tool/parser.rs` is a module of a binary,
  not a binary, and stays in scope.
- A source root nested inside another no longer causes every file beneath it to
  be walked, read, parsed and reported twice. `TargetRootCollector` contributes
  each target's parent directory, so a crate with a lib and a `[[bin]]` under
  `src/bin/` yielded both `src` and `src/bin`; the walk recurses, so everything
  under `src/bin` was reached through both. Nested roots are now dropped.
  Nesting is compared component-wise rather than as text, so a sibling directory
  named `src_generated` is not mistaken for something inside `src`.

Measured on a 312-function crate with two binaries: 76 reported gaps, of which
one was a duplicate of another and one was a `fn main`.

## [0.2.0] - 2026-08-17

### Fixed

- Files whose only declarations are `const` or `static` are now excluded as
  definition-only. A module of lookup tables was previously reported as missing
  a mirrored test, even though `docs/ADRs/ADR-StructuralExclusionsOverSemanticImportance.md`
  cites "a 200-line const table needs no test" as a motivating case: `const`
  and `static` were treated as ignorable filler that could accompany a type
  declaration but never qualify a file on its own. A file whose top-level items
  are all `use` statements is still reported — it declares nothing, so the new
  allowance does not reach it.

### Changed

- A top-level macro invocation or verbatim item now keeps a file in scope.
  Both were previously ignorable, so a file pairing a `struct` with an opaque
  macro body was excluded without the analyzer ever seeing what that macro
  expanded to. A macro-only file was already reported; the change is to files
  that mix a macro with declarations.
- `OPEN_POINTS.md` moved to `docs/OPEN_POINTS.md`, alongside the other
  long-form documentation. The published 0.1.0 tarball carries it at the root.

## [0.1.0] - 2026-08-16

First standalone release. The tool previously lived inside a private workspace
as `etheram-test-gap-gate`; this extracts it as a general-purpose cargo
subcommand with no ties to the repository it grew up in.

### Added

- `cargo twin4rust` reports source files under `src/` that have no mirrored
  test file at the corresponding path under `tests/`.
- `--manifest-path` and repeatable `--package` selection, resolved through
  cargo workspace metadata. Only production targets contribute source roots;
  `test`, `bench`, `example` and `custom-build` targets are excluded.
- Exit code `1` when any mirror is missing and `0` otherwise, for direct use
  as a CI gate.
- Exclusions for `lib.rs`, `main.rs`, `build.rs`, `mod.rs`, definition-only
  files, single-type files whose only `impl` is a trivial `new`, and packages
  whose name ends in `-validation`.

### Changed

- Relicensed from Apache-2.0 to MIT, matching the other published tools in
  this family.
