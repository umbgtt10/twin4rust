# Rules

The complete policy: which files `twin4rust` expects a mirrored test for, where
it expects to find it, and every reason it stays quiet. This is the canonical
description — the README is a summary of it, and `docs/ARCHITECTURE.md`
describes the code that implements it.

---

## The question being asked

`twin4rust` does not measure line coverage, does not measure branch coverage,
and has no opinion on whether existing tests are good. It answers one
structural question:

> For a production source file that should have a mirrored test, does the
> expected mirrored test file exist?

That question is worth asking on its own precisely because coverage cannot
answer it. A file can sit at 90% coverage purely as collateral from an
end-to-end test three layers up, with nothing anywhere naming it as its
subject. Coverage calls that covered. This tool calls it untested, which is the
more useful answer the day it breaks and no failing test points at it.

---

## The mirror rule

For a source file under:

```text
src/<path>/<name>.rs
```

the expected test path is:

```text
tests/<path>/<name>_tests.rs
```

| Source file | Expected mirrored test file |
|---|---|
| `src/raft_node.rs` | `tests/raft_node_tests.rs` |
| `src/implementations/client.rs` | `tests/implementations/client_tests.rs` |
| `src/state/storage/storage_query.rs` | `tests/state/storage/storage_query_tests.rs` |

Existence is the whole check. The tool never opens the test file, so it has no
view on whether the tests inside it are meaningful — see
`docs/ADRs/ADR-MirroredPathIsTheWholeContract.md`.

---

## The analysis order

Per source file, in this order. The first rule that fires ends the check.

1. package is `*-validation` and the file is under `src/` → ignored
2. file is an entry point — `src/lib.rs`, `src/main.rs`, `build.rs`, or a file
   directly under `src/bin/` → ignored
3. file is a `mod.rs` whose top-level items are all imports → ignored
4. file is definition-only → ignored
5. no expected test path could be derived → ignored
6. expected test path exists → satisfied
7. otherwise → **reported**

---

## Exclusions

### 1. Validation crates

If a package name ends in `-validation`, its `src/` tree is skipped entirely.
Such crates hold harness code whose subject is a running system, not a file.

Applied twice, deliberately: `ManifestResolver` drops the `src` source root at
resolution time, and `Analyzer` re-checks per file. The second check catches a
`src/`-relative path arriving from a source root the first did not filter.

### 2. Entry points

`src/lib.rs`, `src/main.rs` and `build.rs` are ignored. They declare structure
or bootstrap a process; neither has a meaningful mirrored subject.

So is any file **directly** under `src/bin/`. That is where cargo looks for a
package's extra binaries, so `src/bin/board_ctl.rs` is an entry point in exactly
the sense `src/main.rs` is, and a mirrored test for its `fn main` would assert
nothing.

The depth matters. `src/bin/tool/main.rs` is an entry point too — not by this
rule, but because `TestFileResolver` derives no path for any file named
`main.rs`. Its sibling `src/bin/tool/parser.rs` is an ordinary module of that
binary, carries ordinary behaviour, and stays in scope.

### 3. `mod.rs` files

`mod.rs` never produces a report.

Two separate mechanisms combine here. `Analyzer` short-circuits a `mod.rs`
whose top-level items are all `use`, `extern crate` or bodiless `mod`
declarations. Independently, `TestFileResolver` returns no expected path for
any file named `mod.rs`, so even a behaviour-bearing one cannot be reported.
The first is a fast path; the second is what actually makes the rule total.

### 4. Definition-only files

A file is definition-only when every top-level non-test item is one of:

- `struct`, `enum`, `type`, `trait`, `const`, `static` — and at least one such
  item is present
- `use`, `extern crate`, bodiless `mod`, which are ignorable rather than
  qualifying
- a **trait impl carrying no methods**, such as `impl Marker for T {}` or a
  blanket `impl<T> Alias for T where ...`

An empty trait impl introduces no executable behaviour, so a mirrored test
could only restate what the compiler already proved. A module of nothing but
lookup tables is the same case: a `const` array has no branch to exercise.

A file with no top-level items at all is *not* definition-only, and is
reported. An empty file is more likely an accident than a decision. A file of
nothing but `use` statements is reported for the same reason — it declares
nothing, so there is nothing to call inert.

A top-level **macro invocation or verbatim item keeps the file in scope**,
whatever else the file contains. The unexpanded form says nothing about the
behaviour it expands to, so one opaque item disqualifies the whole file.

Items carrying a `#[cfg(test)]` attribute are stripped before any of this is
evaluated.

### 5. Single-type files with a trivial constructor

A file is also excluded when it holds:

1. exactly one primary `struct` or `enum`
2. no other top-level behaviour
3. exactly one inherent `impl` block, for that same type
4. containing exactly one method, which:
   - is named `new`
   - has no generics
   - returns `Self`
   - has a body that is a single struct-construction expression
   - has no branching, no loops, no helper calls
   - is not part of a trait implementation

Pure data holders exposing only a trivial constructor do not earn a test file.

```rust
pub struct PeerRecoveryStatus {
    pub height: Height,
    pub last_hash: Hash,
}

impl PeerRecoveryStatus {
    pub fn new(height: Height, last_hash: Hash) -> Self {
        Self { height, last_hash }
    }
}
```

---

## What stays in scope

The exclusions are narrow on purpose. All of these are reported:

- free functions
- a type with two or more methods
- a `new` with any branching, loop or helper call
- a trait impl that carries at least one method
- any file mixing definitions with behaviour

A file with one type plus one impl is **not** automatically excluded. It leaves
scope only by satisfying the trivial-constructor rule exactly.

---

## Package resolution

Packages come from `cargo metadata`, run with `--no-deps`.

1. if `--package` is given one or more times, exactly those packages are analyzed
2. otherwise, if the manifest has a single root package, that one is analyzed
3. otherwise the run fails: a workspace with several members must name one

Only production targets contribute source roots. These target kinds are
excluded: `test`, `bench`, `example`, `custom-build`. These are included:
`lib`, `bin`, `proc-macro`, `rlib`, `dylib`, `cdylib`, `staticlib`.

Each surviving target contributes its source file's parent directory. A package
whose targets yield nothing falls back to `<manifest_dir>/src`.

A root that lies **inside** another is dropped, because the walk recurses and
would otherwise reach every file beneath it twice — reading, parsing and
reporting each one once per containing root. This is the ordinary shape of a
crate with a `[[bin]]` under `src/bin/`, which contributes `src/bin` alongside
the `src` its lib already contributed. Nesting is compared component-wise, so a
sibling named `src_generated` is not mistaken for something inside `src`.

> **The mirror rule assumes a `src/`-rooted layout, and source-root discovery
> does not.** A target rooted outside `src/` — a `[[bin]]` at
> `path = "tools/x.rs"`, say — contributes a source root that is walked and
> classified, but no mirror path can be derived for it, so it is silently never
> reported. Relatedly, a package with no `src/`-rooted target never walks `src/`
> at all, because the fallback fires only when the collected set is empty. Both
> are recorded in [OPEN_POINTS.md](OPEN_POINTS.md); today the tool is only
> trustworthy on packages whose production sources live under `src/`.

---

## Choosing what to point it at

Only pass packages whose test tree takes **files** as its subject.

A repository running a tiered test ladder has trees with different subjects,
and the mirrored-file question is meaningful for exactly one of them:

| Tree | Subject |
|---|---|
| `node/tests` | files |
| `validation/tests/single_node` | clusters of one node |
| `validation/tests/cluster` | clusters of more than one node |

`node/tests/protocol/consensus_tests.rs` and
`validation/tests/single_node/consensus_tests.rs` can both exist and both be
correct. One covers the file `consensus.rs`; the other covers a one-node
cluster reaching consensus. They are not duplicates and neither substitutes for
the other.

Point this tool at a cluster-level harness and every one of its sources is
reported, because by design none of them has a mirror. The `-validation` rule
covers the common naming convention; anything else is the caller's choice.

There is deliberately no flag letting one package's tests satisfy another's
expectation — see `docs/ADRs/ADR-NoCrossPackageSatisfaction.md`.

---

## Output

Success:

```text
twin4rust report

All mirrored test-gap expectations are satisfied.
```

Failure:

```text
twin4rust report

Files without a matching mirrored test file:
- my-crate: src/foo.rs -> tests/foo_tests.rs
- other-crate: src/bar/baz.rs -> tests/bar/baz_tests.rs

summary: packages_with_gaps=2 missing_files=2
```

Gaps are sorted by package name, then source path, then expected test path, so
the report is stable across runs and diffable in CI.

Exit code is `1` when any gap is reported and `0` otherwise.

---

## Maintenance

If the mirrored-test policy changes, update together:

1. this document
2. the tool logic
3. the integration tests
4. any wrapper script, but only if package selection or invocation changes
