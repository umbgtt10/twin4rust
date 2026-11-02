# ADR-AMirrorMustBeEarned

- **Status:** Accepted
- **Date:** 2026-11-02

## Context
A file that needs a mirrored test must have one; the converse matters as much. A
file of definitions or a data holder needs none -- and a mirror for it can only
test what
the compiler already proves: that a derived `Default` gives zeros, that a clone
equals its original, that a constructor keeps its arguments, that a boxed trait
object dispatches. Such tests add to the count without adding coverage, and the
exemption that says the file has nothing to test is contradicted by a file of
tests for it.

## Decision
A mirrored test file for a source that earns none -- a file of definitions, or a
data holder -- is reported, beside the missing mirrors, and fails the run.

## Forcing constraints / Evidence
On `etheram-ibft` at 2026-11-02, 18 mirrors held 73 tests of this kind: derived
`Clone` and `PartialEq` on certificates, constructors keeping their arguments, a
trait object forwarding to a recording fake.

## Rejected alternatives
- **Reporting a mirror of every exempt file.** A humble adapter's forwarding is
  real code, and a trait's default method is behaviour: a mirror of either may
  test something, so for them a mirror is allowed rather than demanded or
  refused. Entry points, `mod.rs` and `*-validation` sources have no mirror path.
- **Opening the test file to see what it tests.** The contract is the path
  ([ADR-MirroredPathIsTheWholeContract](ADR-MirroredPathIsTheWholeContract.md));
  the source alone decides whether a mirror is earned.

## Consequences
A codebase holding such tests sees them reported on its first run, and removes
them or moves what they check to the tests of the code that uses the record.
The report lists them in a section of their own and counts them in the
summary's `unearned_mirrors=`, both present only when there is one: a run
without unearned mirrors prints the report it always prints.

## Enforcement
`DefinitionAnalyzer::mirror_expectation` answers `Required`, `Optional` or
`Unearned` for each file; `Analyzer` reports an `UnearnedMirror` for a mirror of
an `Unearned` one; `AnalysisReport::is_empty` counts both kinds, so the exit code
does.

## Related
- [ADR-RecordsFollowStern4rust](ADR-RecordsFollowStern4rust.md)
- [ADR-MirroredPathIsTheWholeContract](ADR-MirroredPathIsTheWholeContract.md)
- [ADR-HumbleAdaptersAtUntestableBoundaries](ADR-HumbleAdaptersAtUntestableBoundaries.md)
- `docs/RULES.md`, "Mirrors a source does not earn"
