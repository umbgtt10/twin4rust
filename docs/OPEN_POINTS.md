# Open Points

Known gaps, deliberately recorded rather than silently assumed correct. Each
entry states what was actually observed, not what is suspected.

## Source roots outside `src/` are walked but can never be reported

This is the *outside*-`src/` half of target-driven root discovery. The
*inside*-`src/` half — a `[[bin]]` under `src/bin/` contributing `src/bin`
alongside `src`, so every file beneath it was walked and reported twice — was
fixed in 0.2.1 by dropping any root nested inside another. That fix does not
help here: a root outside `src/` is not nested in anything, so it survives
pruning and still cannot produce a mirror path.

`TargetRootCollector` derives source roots from each production target's own
`src_path` parent, so a `[[bin]]` declared at `path = "tools/probe.rs"`
contributes `tools/` as a source root and `SourceWalker` duly walks it. But
`TestFileResolver::expected_test_file` computes the mirror by
`source_file.strip_prefix(manifest_dir/src)`, which fails for any path outside
`src/`. The resolver returns `None`, and `Analyzer::check_source_file` treats
`None` as "no expectation" — indistinguishable, at that point, from a
deliberately excluded file.

Observed on a two-target crate (`[lib] path = "src/lib.rs"` plus
`[[bin]] path = "tools/probe.rs"`), both files carrying a branching free
function and neither having a mirror:

```text
twin4rust report

Files without a matching mirrored test file:
- probe2: src/inner.rs -> tests/inner_tests.rs

summary: packages_with_gaps=1 missing_files=1
```

`tools/probe.rs` is absent. It was read, parsed and classified as
behaviour-bearing; only the final path mapping dropped it.

This is the worst-shaped failure mode a gate can have — it under-reports
silently, and the run exits `0` on a file the tool examined and had an opinion
about. Anyone whose crate keeps a binary outside `src/` is getting less
coverage from this gate than the clean report implies.

Not started. The fix is not simply relaxing the `strip_prefix`: the mirror rule
has to gain a notion of "which root is this file relative to", which is the same
work `docs/ROADMAP.md` Phase 5 scopes as multiple test roots. Until then, the
honest statement is that the mirror rule assumes a `src/`-rooted layout and the
source-root discovery does not.

### Related: a package with no `src/`-rooted target never walks `src/` at all

The same probe with only the `tools/` bin declared, and no `[lib]`, reports
nothing for a behaviour-bearing `src/inner.rs`. `TargetRootCollector::ensure_fallback`
adds `<manifest_dir>/src` only when the collected set is *empty*; here it is
non-empty (it holds `tools/`), so the fallback never fires and `src/` is never
walked. Same root cause — target-driven discovery disagreeing with a hardcoded
mirror rule — recorded separately because the mechanism differs.

## The `mod.rs` rule is expressed twice, and the `Analyzer` branch cannot change the outcome

`Analyzer::should_ignore_source_file` short-circuits a `*/mod.rs` whose
top-level items are all imports, via `DefinitionAnalyzer::mod_file_is_import_only`.
Independently, `TestFileResolver::expected_test_file` returns `None` for any
file named `mod.rs`.

The second rule subsumes the first. A `mod.rs` that is *not* import-only falls
through the `Analyzer` branch, gets parsed a second time by
`is_definition_only_source`, and is then dropped anyway when the resolver
declines to produce a path. Observed on a `src/feature/mod.rs` carrying two
branching functions and no mirror: reported as satisfied, exit `0`.

So the `Analyzer` branch costs an extra `parse_file` on exactly the files it
cannot affect, and — more importantly — it advertises a policy the tool does not
implement. A reader of `analyzer.rs` alone would reasonably conclude that a
behaviour-bearing `mod.rs` *is* reported.

Not started. Resolving it is a choice, not a cleanup: either delete the
`Analyzer` branch and document that `mod.rs` is unconditionally exempt, or
delete the `TestFileResolver` special case and let a behaviour-bearing `mod.rs`
be reported. `docs/RULES.md` currently documents the observed behaviour (never
reported) rather than the apparent intent, so the docs are correct either way,
but they describe a rule implemented in a confusing place.

## Every file is read and parsed before its mirror's existence is checked

`Analyzer::check_source_file` reads the source, runs the `mod.rs` check, and
runs the full `syn` definition-only classification — and only then asks
`TestFileResolver` for a path and calls `exists()` on it.

In a healthy repository the overwhelming majority of files have their mirror,
so the parse is performed and discarded for almost every file analyzed. Moving
the existence check ahead of the read would skip both the I/O and the parse for
that whole majority, leaving the classification to run only on files that are
about to be reported.

Not started, and not urgent: analysis of a mid-sized workspace is already
sub-second. Recorded because the current ordering is the exact inverse of the
cheap-check-first shape, and that is a deliberate-looking choice that was not
in fact deliberate.

## `analysis_report_tests.rs` is narrower than its contents

The file mirrors `src/analysis_report.rs`, which is what the gate requires, but
it also exercises `Config`, `MissingTestGap` and `PackageContext`. Those three
are definition-only types, so the gate demands no mirror for them and their
coverage has nowhere else it is obliged to live.

The name is therefore accurate about its obligation and misleading about its
contents. Splitting it three ways would produce files whose subjects the gate
explicitly does not require, which is its own kind of noise.

Not started; flagged so the next person to add a test for one of those three
types knows why it is sitting in a file named after a different one.

## Structural classification cannot see through macros or type aliases

Per `docs/ADRs/ADR-StructuralExclusionsOverSemanticImportance.md`, exclusions
are decided from the `syn` AST with no type resolution. A data holder generated
by a derive or declarative macro, or one whose `new` delegates to a helper for
readability, does not match the trivial-constructor shape and is reported. A
top-level macro invocation is treated the same way deliberately: it is not an
ignorable item, so its mere presence keeps the file in scope.

This is the intended direction of error — ambiguity resolves toward visible
rather than silent — and the alternative costs the ability to run on a tree that
does not compile. Recorded here so a report on a macro-heavy crate is
recognized as this known limit rather than investigated as a bug.

No action planned. Closing it would mean adopting type resolution, which the
ADR rejects on stated grounds.
