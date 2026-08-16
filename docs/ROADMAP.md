# twin4rust Roadmap

This document tracks the planned evolution of `cargo-twin4rust` beyond the
currently shipped release.

For what is available today, see
[IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md). For released versions, see
[CHANGELOG.md](../CHANGELOG.md).

## Product Direction

`twin4rust` aims to be the structural half of a test-quality story: it answers
whether a file has a test whose subject is that file, and leaves how well that
test exercises it to coverage tooling.

The long-term direction is:

- a mirror rule that survives contact with real project layouts
- exclusions narrow enough to trust, and explicable in one sentence each
- low-friction Cargo integration
- stable machine-readable output
- a reusable library surface for embedding

## Guiding Principles

- Existence of the mirror is the contract; never grade the test file's contents
- Prefer a visible ambiguous file over a silent exclusion
- Structural classification only — no type resolution, no build step
- One metric per tool: coverage belongs to `crap4rust`, duplication to
  `dry4rust`
- Add configurability only after the defaults are shown to be wrong somewhere real

## Current Baseline

The shipped release provides `cargo twin4rust` as a published cargo subcommand,
the mirrored-path check with its five exclusion categories, production-target
source-root resolution, stable sorted reporting, and CI-ready exit codes. See
[IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md) for the full list.

## Planned Phases

### Phase 2: Machine-Readable Output

Goal: make the report consumable by something other than a human reading a
terminal.

Planned scope:

- `--output-format json` carrying the same `AnalysisReport` shape the text
  renderer projects from
- an output-file option instead of stdout-only
- a documented, versioned schema

Exit criteria:

- a CI job can diff two runs and report only newly introduced gaps

### Phase 3: Baselines and Adoption Paths

Goal: let a project with existing gaps adopt the gate without a flag day.

Planned scope:

- a baseline file recording currently-accepted gaps
- fail only on regressions against that baseline
- an explicit, reviewable way to retire baseline entries

Exit criteria:

- a large existing codebase can turn the gate on in one commit and burn the
  baseline down over time

### Phase 4: Configuration

Goal: stop requiring every preference to travel through argv.

Planned scope:

- `twin4rust.toml` for package selection and exclusions
- per-workspace and per-crate defaults
- suppression entries that carry a required reason
- a configurable mirror rule, for projects whose test layout is not `tests/`

Exit criteria:

- project policy lives in a versioned file rather than a shell script

### Phase 5: Layout Fidelity

Goal: handle the layouts the current single rule does not.

Planned scope:

- multiple test roots per package
- alternative suffixes to `_tests.rs`
- source roots outside `src/`, resolved from target metadata rather than assumed
- a documented answer for crates that deliberately colocate tests

Exit criteria:

- the tool is useful on a project that did not adopt this exact convention first

### Phase 6: Library Surface

Goal: make `twin4rust` embeddable, not only runnable.

Planned scope:

- a stable public API over `Analyzer` and its report types
- extension points for the mirror rule and the exclusion set
- editor and code-scanning adapters where they prove necessary

Exit criteria:

- a third-party tool can embed the analysis without shelling out

## Deferred Ideas

Intentionally not prioritized until the core is further along:

- grading the contents of test files, which
  [ADR-MirroredPathIsTheWholeContract](ADRs/ADR-MirroredPathIsTheWholeContract.md)
  rules out by design
- any form of cross-package satisfaction, which
  [ADR-NoCrossPackageSatisfaction](ADRs/ADR-NoCrossPackageSatisfaction.md)
  rules out on evidence
- type resolution to sharpen classification, at the cost of only running on a
  compiling tree
- a plugin architecture before the data model has stabilized

## Success Measure

The roadmap is succeeding if each phase improves one of these:

- fewer false positives without adding a silent exclusion
- easier adoption on a codebase that did not start with this convention
- clearer automation surfaces
- broader reuse without destabilizing the default CLI workflow

## Revision Policy

This roadmap is directional, not contractual. Phases may be reordered or
narrowed if real use shows a smaller, sharper scope is the better engineering
decision.
