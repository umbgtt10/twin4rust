# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

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
