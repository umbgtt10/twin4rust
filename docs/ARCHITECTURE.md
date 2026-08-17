# Architecture

How a `cargo twin4rust` invocation flows through the code. This is a map of
what exists today, not a decision record — see `docs/ADRs/` for the "why"
behind these shapes, and `docs/RULES.md` for the policy they implement.

---

## Pipeline

```
Args (clap)
  → Runner::run(args)
      1. Config { manifest_path, packages }
      2. ManifestResolver::new(config).resolve() -> Vec<PackageContext>
           cargo metadata --no-deps
           select requested packages, or the single root package
           per package: TargetRootCollector -> source roots
           drop the src root of any *-validation package
      3. Analyzer::analyze_packages(&packages) -> Vec<AnalysisReport>
           per package, per source root:
             SourceWalker::walk    -> every .rs file beneath it
             per file: exclusion rules, then TestFileResolver
             collect MissingTestGap for each survivor whose mirror is absent
      4. ReportPrinter::print(&reports)
      5. exit(1) if any report is non-empty
```

Errors — an unreadable manifest, an unknown package name, a source file that
fails to read or parse — propagate out of `Runner::run` as `anyhow::Error` and
surface through `main`. That is distinct from a *successful* run that found
gaps and exits `1`.

## Components

| Type | File | Responsibility |
|---|---|---|
| `Args` | `args.rs` | clap parsing, plus the cargo-subcommand argv fixup |
| `Config` | `config.rs` | plain-data form of the parsed arguments |
| `Runner` | `runner.rs` | wires resolution → analysis → reporting, owns the exit code |
| `ManifestResolver` | `manifest_resolver.rs` | `cargo metadata`, package selection, `PackageContext` construction, path relativization |
| `TargetRootCollector` | `target_root_collector.rs` | production-target filtering and source-root deduplication |
| `SourceWalker` | `source_walker.rs` | recursive `.rs` discovery beneath one source root |
| `Analyzer` | `analyzer.rs` | applies the exclusion rules and collects gaps |
| `DefinitionAnalyzer` | `definition_analyzer.rs` | `syn` classification: definition-only, import-only `mod.rs`, `#[cfg(test)]` stripping |
| `TrivialConstructorDetector` | `trivial_constructor_detector.rs` | the single-type-plus-trivial-`new` rule |
| `HumbleAdapterDetector` | `humble_adapter_detector.rs` | the same rule one method further along: forwarding methods |
| `BehaviourlessImplDetector` | `behaviourless_impl_detector.rs` | whether one `impl` block carries executable behaviour |
| `TestFileResolver` | `test_file_resolver.rs` | `src/<path>/<name>.rs` → `tests/<path>/<name>_tests.rs` |
| `ReportPrinter` | `report_printer.rs` | renders the report, sorted and stable |

## Data model

| Type | Scope | Carries |
|---|---|---|
| `PackageContext` | one resolved package | name, manifest dir, source roots |
| `MissingTestGap` | one reported file | package name, relative source file, expected test file |
| `AnalysisReport` | one analyzed package | package name plus its `Vec<MissingTestGap>` |

All three are plain data. `AnalysisReport::is_empty` is what `Runner` folds
over to decide the exit code, and what `ReportPrinter` counts for
`packages_with_gaps`.

## Why the behaviourless-impl question is its own type

Both the definition-only rule and the trivial-constructor rule need to know
whether an `impl` block carries a method, and only the first of them used to
have the answer. The second admitted `Item::Impl(_)` outright, so a file holding
a trivial `new` was exempt whatever else its impls did — the shape of every
adapter behind a seam. Sharing one detector is what makes the two rules agree on
the same definition of behaviour rather than each carrying its own.

The definition is asymmetric on purpose: a *trait* impl with no methods is
behaviourless, an *inherent* impl is not, even when empty. An inherent impl is
the block the trivial-constructor and humble-adapter rules count, and calling an
empty one inert would let them admit a second one while still claiming the file
holds exactly one.

## Analysis is AST-structural

`DefinitionAnalyzer` and `TrivialConstructorDetector` work on a `syn::File`
parsed from the source text. There is no type resolution and no build step, so
the tool runs against any syntactically valid source whether or not the crate
compiles, and whether or not its dependencies are vendored.

The cost is that classification cannot see through a type alias or a macro
expansion. That is a deliberate trade — see
`docs/ADRs/ADR-StructuralExclusionsOverSemanticImportance.md`.

## Paths

`ManifestResolver::relative_file` normalizes separators to `/` when relativizing
against the manifest directory, so a report generated on Windows is identical to
one generated on Linux. Every path in the output has been through it.

## CLI layer

`Args::parse_args` routes argv through `Args::without_cargo_subcommand` before
clap sees it. Cargo invokes `cargo twin4rust ...` as
`cargo-twin4rust twin4rust ...`, so the subcommand name arrives as an extra
leading argument that clap would reject; running the binary directly does not
repeat it, so the strip is conditional on `argv[1]` and nothing else. See
`docs/ADRs/ADR-CargoSubcommandPackaging.md`.

## Related

- `docs/RULES.md` — the mirror rule and every exclusion, in full
- `docs/ADRs/` — why the contract is a path, why there is no cross-package flag,
  why exclusions are structural, and why the crate is named `cargo-twin4rust`
- `docs/ROADMAP.md` — what ships today and what comes next
