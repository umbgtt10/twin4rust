# Architecture

How a `cargo twin4rust` invocation flows through the code. This is a map of
what exists today, not a decision record — see `docs/ADRs/` for the "why"
behind these shapes, and `docs/RULES.md` for the policy they implement.

The workspace has two published crates. `core/` (`twin4rust-core`, library
`twin4rust`) is the analysis: `#![no_std]`, no technology, every piece of it it
needs behind a trait. `cli/` (`cargo-twin4rust`, library `twin4rust_cli`) is the
command: argument parsing, the composition root, and the adapters that give the
core's traits the real cargo metadata and the real file system. See
`docs/ADRs/ADR-TechnologyFreeCore.md`.

---

## Pipeline

```
cli/src/main.rs
  → EntryPoint::run(argv)                                    (cli)
      Args::without_cargo_subcommand, then clap
      CargoWorkspaceReader + WalkdirSourceTree + TextReportRenderer
  → Runner::new(&reader, &tree, &renderer).run(&config)      (core)
      1. ManifestResolver::resolve(&config) -> Vec<PackageContext>
           WorkspaceReader::describe          -> WorkspaceDescription
           select requested packages, or the single root package
           per package: TargetRootCollector -> source roots
           drop the src root of any *-validation package
      2. Analyzer::analyze_packages(&packages) -> Vec<AnalysisReport>
           per package, per source root:
             SourceTree::rust_files   -> every .rs file beneath it
             per file: SourceTree::read_source, exclusion rules,
               then TestFileResolver
             SourceTree::contains(mirror)? else a MissingTestGap
      3. ReportRenderer::render(&reports) -> String
      -> RunOutcome { output, has_gaps }
  → print!(output); exit code 1 if has_gaps, else 0          (cli)
```

Errors — an unreadable manifest, an unknown package name, a source file that
fails to read or parse — come out of `Runner::run` as a typed `RunFailure`,
which keeps the failure's cause, and surface through `main` as the same chain
anyhow always printed. That is distinct from a *successful* run that found gaps
and exits `1`.

## Ports

Each is a trait in `core/src/traits/`, `Sync` so a `Runner` holding them stays
`Send`, with an in-memory fake under `core/tests/support/` and a real adapter in
`cli/src/system/`:

| Port | Answers | Adapter in `cli` |
|---|---|---|
| `WorkspaceReader` | the workspace's packages and their targets | `CargoWorkspaceReader`: `cargo metadata --no-deps`, translated into the core's records |
| `SourceTree` | the `.rs` files under a root, a file's text, whether a path exists | `WalkdirSourceTree`: walkdir and `std::fs` |
| `ReportRenderer` | the reports as text | none — `TextReportRenderer` is the core's own |

## Components

| Type | File | Responsibility |
|---|---|---|
| `Args` | `cli/src/invocation/args.rs` | clap parsing, plus the cargo-subcommand argv fixup |
| `EntryPoint` | `cli/src/invocation/entry_point.rs` | the composition root: argv in, report on stdout, exit code back |
| `CargoWorkspaceReader` | `cli/src/system/cargo_workspace_reader.rs` | `cargo metadata`, translated into `WorkspaceDescription` |
| `WalkdirSourceTree` | `cli/src/system/walkdir_source_tree.rs` | recursive `.rs` discovery, reading, existence on disk |
| `PortablePath` | `cli/src/system/portable_path.rs` | a platform path into the core's `/`-only `SourcePath` |
| `Config` | `core/src/config.rs` | the packages a run was asked for |
| `Runner` | `core/src/runner.rs` | wires resolution → analysis → rendering, returns the verdict |
| `ManifestResolver` | `core/src/manifest_resolver.rs` | package selection, `PackageContext` construction, path relativization |
| `TargetRootCollector` | `core/src/target_root_collector.rs` | production-target filtering and source-root deduplication |
| `Analyzer` | `core/src/analyzer.rs` | applies the exclusion rules and collects gaps, through the `SourceTree` |
| `DefinitionAnalyzer` | `core/src/definition_analyzer.rs` | `syn` classification: definition-only, import-only `mod.rs`, `#[cfg(test)]` stripping |
| `TrivialConstructorDetector` | `core/src/trivial_constructor_detector.rs` | the single-type-plus-trivial-`new` rule |
| `HumbleAdapterDetector` | `core/src/humble_adapter_detector.rs` | the same rule one method further along: forwarding methods |
| `BehaviourlessImplDetector` | `core/src/behaviourless_impl_detector.rs` | whether one `impl` block carries executable behaviour |
| `TestFileResolver` | `core/src/test_file_resolver.rs` | `src/<path>/<name>.rs` → `tests/<path>/<name>_tests.rs` |
| `TextReportRenderer` | `core/src/text_report_renderer.rs` | renders the report, sorted and stable |
| `SourcePath` | `core/src/workspace/source_path.rs` | a path held as text, `/` its only separator, compared by component |

## Data model

| Type | Scope | Carries |
|---|---|---|
| `WorkspaceDescription` | one workspace | its packages, and the root package's name if cargo resolved one |
| `PackageDescription` | one package | name, manifest path, targets |
| `TargetDescription` | one target | its kinds, its source path |
| `PackageContext` | one resolved package | name, manifest dir, source roots |
| `MissingTestGap` | one reported file | package name, relative source file, expected test file |
| `AnalysisReport` | one analyzed package | package name plus its `Vec<MissingTestGap>` |
| `RunOutcome` | one run | the rendered report and whether any mirror is missing |
| `ConstructorScanState` | one trivial-constructor scan | the one type seen so far, and how many trivial-`new` impls name it |

The descriptions, `PackageContext`, `MissingTestGap`, `RunOutcome` and
`ConstructorScanState` are plain data: records, their fields `pub`, built by
literal. `AnalysisReport` is an object -- its fields are private, read through
`package_name()` and `missing()` -- because it answers questions about them:
`AnalysisReport::is_empty` is what `Runner` folds over to decide the verdict,
and what `TextReportRenderer` counts for `packages_with_gaps`.
`ConstructorScanState` is what `TrivialConstructorDetector` threads through its
walk over a file's items, each step handing back the updated state rather than
writing it through a `&mut` parameter. Nothing else uses it, so it is a private
struct in `trivial_constructor_detector.rs` rather than a module of its own.

The failures are records too: `SourceReadFailure` (a file the tree could not
list or read), `WorkspaceReadFailure` (a workspace that could not be described),
`ParseFailure` (a source syn could not parse), and `RunFailure`, one variant per
way a run fails, wrapping the others with their causes.

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

Every path the core sees is a `SourcePath` whose only separator is `/`: `cli`'s
`PortablePath` turns the platform's into it where cargo's and the walker's paths
come in, and a `.rs` file whose path is not UTF-8 fails the run, named, rather
than being reported under a lossy name. `ManifestResolver::relative_file`
relativizes against the manifest directory, so a report generated on Windows is
identical to one generated on Linux. Every path in the output has been through
it.

## CLI layer

`EntryPoint` routes argv through `Args::without_cargo_subcommand` before clap
sees it. Cargo invokes `cargo twin4rust ...` as
`cargo-twin4rust twin4rust ...`, so the subcommand name arrives as an extra
leading argument that clap would reject; running the binary directly does not
repeat it, so the strip is conditional on `argv[1]` and nothing else. See
`docs/ADRs/ADR-CargoSubcommandPackaging.md`.

## Related

- `docs/RULES.md` — the mirror rule and every exclusion, in full
- `docs/ADRs/` — why the contract is a path, why there is no cross-package flag,
  why exclusions are structural, why the crate is named `cargo-twin4rust`, and
  why the analysis is a technology-free crate of its own
- `docs/ROADMAP.md` — what ships today and what comes next
