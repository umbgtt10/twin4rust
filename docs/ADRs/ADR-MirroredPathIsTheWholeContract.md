# ADR-MirroredPathIsTheWholeContract

- **Status:** Accepted
- **Date:** 2026-08-16

## Context

A tool that reports "this file is untested" invites a much larger tool. It
could open the test file and count assertions, check that the test names the
source file's types, measure how much of the file the test actually reaches, or
score whether the assertions are meaningful.

Every one of those is a judgement the tool can be wrong about, and a false
positive in a build gate is far more expensive than a missed finding: it trains
the team to add `--allow` flags until the gate stops meaning anything.

## Decision

The contract is a path. `src/<path>/<name>.rs` expects
`tests/<path>/<name>_tests.rs` to exist, and `twin4rust` never opens that file.

## Forcing constraints / Evidence

Existence is the only property that is both objective and cheap. It has no
false positives that are not also policy disagreements — if the file is there
and empty, the tool is still factually correct that the mirror exists, and the
emptiness is visible to any reviewer in a way a coverage percentage is not.

It is also the property line coverage structurally cannot report. A file sitting
at 90% purely as collateral from an end-to-end test three layers up is
indistinguishable, in a coverage report, from one with a dedicated test suite.
The mirrored-path question separates them.

## Rejected alternatives

- **Parse the test file and require it to reference the source type.** Breaks
  on re-exports, generic helpers and any test that drives the subject through a
  builder. Punishes indirection that is otherwise good design.
- **Require a minimum assertion count.** Trivially gamed, and wrong for files
  whose contract is one equality.
- **Fold in line coverage.** That is `crap4rust`'s job. Two tools with one
  metric each stay explicable; one tool with a blended score does not.

## Consequences

The tool cannot detect an empty or bad mirrored test file. It is a structural
gate, not a quality gate, and its output should be read that way: a clean run
means every file has a place for its tests, not that the tests are good.

It also means the check costs one `Path::exists` per surviving file, so the
whole run is dominated by parsing rather than I/O.

## Enforcement

`TestFileResolver::expected_test_file` returns a path and nothing else; the
only consumer is an `exists()` call in `Analyzer::check_source_file`. There is
no code path that reads a test file.

## Related

- [ADR-NoCrossPackageSatisfaction](ADR-NoCrossPackageSatisfaction.md)
- [ADR-StructuralExclusionsOverSemanticImportance](ADR-StructuralExclusionsOverSemanticImportance.md)
- `docs/RULES.md`
