# twin4rust

[![crates.io](https://img.shields.io/crates/v/cargo-twin4rust.svg)](https://crates.io/crates/cargo-twin4rust)
[![docs.rs](https://docs.rs/cargo-twin4rust/badge.svg)](https://docs.rs/cargo-twin4rust)
[![license](https://img.shields.io/crates/l/cargo-twin4rust.svg)](LICENSE)

A source file under `src/` should have a test file mirroring it under `tests/`.
`twin4rust` fails the build when one doesn't.

```text
twin4rust report

Files without a matching mirrored test file:
- node: src/raft_node.rs -> tests/raft_node_tests.rs
- node: src/state/raft_state.rs -> tests/state/raft_state_tests.rs

summary: packages_with_gaps=1 missing_files=2
```

## What it is not

This is not a coverage tool. It does not measure lines, it does not measure
branches, and it has no opinion on whether your tests are any good — it never
opens the test file it is looking for.

It answers one structural question: **does the mirrored test file exist?**

That question is worth asking on its own, because line coverage cannot answer
it. A source file can be at 90% coverage purely as collateral from an
end-to-end test three layers up, with nothing anywhere that names it as its
subject. Coverage reports that as covered. `twin4rust` reports it as untested,
which is the more useful answer when the file later breaks and no failing test
points at it.

## Install

```sh
cargo install cargo-twin4rust
```

## Use

```sh
cargo twin4rust --manifest-path Cargo.toml --package my-crate
```

| Flag | Description |
|---|---|
| `--manifest-path` | Cargo manifest to analyze. Defaults to the manifest in the current directory. |
| `--package` | Package to analyze. Repeatable. If omitted, a single-package manifest is analyzed; a workspace requires at least one. |

Exit code is `0` when every expected mirror exists and `1` when any is missing,
so it drops into CI as-is. Gaps are sorted by package, then source path, so the
output is stable across runs and diffable.

## The mirror rule

```text
src/<path>/<name>.rs   ->   tests/<path>/<name>_tests.rs
```

| Source file | Expected test file |
|---|---|
| `src/raft_node.rs` | `tests/raft_node_tests.rs` |
| `src/state/storage/storage_query.rs` | `tests/state/storage/storage_query_tests.rs` |

**The rule is `src/`-rooted, and that is currently a hard requirement rather
than a default.** A file outside `src/` yields no mirror path, and a file with
no mirror path is not reported — see [Known limitations](#known-limitations).

## What it skips

The gate is deliberately conservative — it would rather stay quiet than force a
test file for something with no behaviour to test.

- **Entry points**: `src/lib.rs`, `src/main.rs`, `build.rs`, and any file
  directly under `src/bin/`. A module *inside* a `src/bin/<name>/` binary stays
  in scope
- **every `mod.rs`**, unconditionally — including one that carries behaviour
- **Definition-only files**: every top-level item is a `struct`, `enum`, `type`,
  `trait`, `const` or `static` declaration, or an ignorable `use`, `extern
  crate` or bodiless `mod` — with at least one declaring kind present. A
  top-level macro invocation keeps the file in scope, since its expansion is
  never seen
- **Method-less trait impls**, such as `impl Marker for T {}` or a blanket
  `impl<T> Alias for T where ...`, which introduce nothing to assert
- **Single-type files whose only `impl` is a trivial `new`** — one method, no
  generics, returns `Self`, body is a single struct literal, no branching, no
  loops, no helper calls. Pure data holders do not earn a test file.
- The `src/` tree of any package whose name ends in `-validation`
- Items carrying `#[cfg(test)]`, stripped before any of the above is evaluated

Anything else stays in scope. A `new` with a branch in it, a second method, or
a trait impl carrying at least one method all put the file back on the list. A
file with no top-level items at all is reported rather than skipped — an empty
source file is more likely an accident than a decision.

## Known limitations

**Files outside `src/` are analyzed and then silently dropped.** Source roots
come from each production target's own path, so a `[[bin]]` at
`path = "tools/x.rs"` contributes `tools/` as a root and the file is walked,
read and classified — but no mirror path can be derived for it, and a file with
no mirror path is treated as having no expectation. It does not appear in the
report and does not affect the exit code.

Relatedly, a package with no target rooted under `src/` never walks `src/` at
all, because the fallback that adds it fires only when no root was collected.

Until this is resolved, **`twin4rust` is only trustworthy on packages whose
production sources live under `src/`.** Both behaviours are recorded in
[docs/OPEN_POINTS.md](docs/OPEN_POINTS.md), along with the ordering and `mod.rs` quirks
behind them.

## Choosing what to point it at

Only pass packages whose test tree takes *files* as its subject.

A repository running a tiered test ladder has trees with different subjects: one
covers files, another covers a single-node cluster, another covers a multi-node
cluster. `node/tests/raft/replication_tests.rs` and
`validation/tests/cluster/replication_tests.rs` can both exist and both be
correct — one covers the file, the other covers a cluster replicating. Neither
substitutes for the other.

Point this tool at a cluster-level harness and every one of its sources is
reported as a gap, because by design none of them has a mirror. The
`-validation` suffix rule covers the common naming convention; anything else is
your call.

There is deliberately no flag letting one package's tests satisfy another's
expectation. Such a flag existed briefly and was removed: aimed at a harness
tree it made the number improve while coverage did not, because a cluster test
was silencing a missing file-level test. If a source file has no test whose
subject is that file, that is the finding — wherever else it happens to be
exercised.

## Documentation

| Document | Contents |
|---|---|
| [docs/RULES.md](docs/RULES.md) | The canonical policy — mirror rule, every exclusion, package resolution, output |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | How an invocation flows through the code |
| [docs/ADRs/](docs/ADRs/README.md) | The load-bearing decisions and why they were forced |
| [docs/IMPLEMENTED-FEATURES.md](docs/IMPLEMENTED-FEATURES.md) | What ships today |
| [docs/ROADMAP.md](docs/ROADMAP.md) | What comes next |
| [docs/OPEN_POINTS.md](docs/OPEN_POINTS.md) | Known gaps, deliberately deferred |
| [CHANGELOG.md](CHANGELOG.md) | Release history |

## Related

- [`cargo-crap4rust`](https://crates.io/crates/cargo-crap4rust) — CRAP scores across Rust crates
- [`slotgate`](https://crates.io/crates/slotgate) — bounded-parallelism job runner giving each slot a disjoint port range

## License

MIT. See [LICENSE](LICENSE).
