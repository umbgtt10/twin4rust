# ADR-NoCrossPackageSatisfaction

- **Status:** Accepted
- **Date:** 2026-08-16

## Context

A repository running a tiered test ladder has several test trees with different
subjects: one covers files, another covers a single-node cluster, another
covers a multi-node cluster. Pointed at such a repository, it is tempting to
let the cluster tree "count" for the file tree — the code *is* exercised, after
all, so why report it.

## Decision

No flag lets one package's tests satisfy another package's mirrored-file
expectation. If a source file has no test whose subject is that file, that is
the finding, wherever else it happens to be exercised.

## Forcing constraints / Evidence

Such a flag existed briefly and was removed. Aimed at a harness tree it made
the number improve while coverage did not: a cluster test was silencing a
missing file-level test, and the gate went green on a repository that had
gained nothing.

The two trees are not substitutes. `node/tests/protocol/consensus_tests.rs` and
`validation/tests/single_node/consensus_tests.rs` can both exist and both be
correct — one covers the file `consensus.rs`, the other covers a one-node
cluster reaching consensus. Neither is redundant with the other, and a flag
that treats them as interchangeable encodes the opposite claim.

## Rejected alternatives

- **`--satisfied-by <package>`.** The flag that was removed. Its failure mode
  was silent and pointed the wrong way: it made the gate more permissive
  exactly where the ladder was weakest.
- **Union every test tree in the workspace before checking.** The same defect
  with no flag to blame it on.

## Consequences

The caller must pass only packages whose test tree takes files as its subject.
Point the tool at a cluster harness and every one of its sources is reported,
because by design none of them has a mirror.

The `-validation` suffix rule covers the common naming convention for such
harnesses. Anything else is the caller's responsibility, which is a real
usability cost accepted deliberately.

## Enforcement

`Analyzer::check_source_file` resolves the expected path from the analyzed
package's own manifest directory. There is no parameter through which another
package's `tests/` tree could enter the check.

## Related

- [ADR-MirroredPathIsTheWholeContract](ADR-MirroredPathIsTheWholeContract.md)
- `docs/RULES.md`
