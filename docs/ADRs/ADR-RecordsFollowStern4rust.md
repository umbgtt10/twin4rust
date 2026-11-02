# ADR-RecordsFollowStern4rust

- **Status:** Accepted
- **Date:** 2026-11-02

## Context
A pure data holder earns no mirrored test. stern4rust's `records-and-objects`
defines one: a type whose inherent methods are getters, plain constructors and
`with_` steps. The two tools run side by side in the same gates, and two
definitions of the same thing would let one tool demand a test for a type the
other calls a record -- and, with
[ADR-AMirrorMustBeEarned](ADR-AMirrorMustBeEarned.md), demand what the other
forbids.

## Decision
A data holder is a single type whose inherent methods are all getters, plain
constructors or `with_` steps, in stern4rust's shapes, beside nothing but
definitions and trait impls carrying no method.

## Forcing constraints / Evidence
The shapes are stern4rust's `MethodClassifier`: a getter takes `&self` alone and
reads one field of `self`, at most through `as_ref`, `as_str`, `clone` and their
kin; a plain constructor takes no receiver, returns its own type and builds it
from bare parameters; a step takes `self` by value and one parameter and replaces
one field, the rest `..self`. Bare parameters are what "no helper calls" means
for a constructor: a field built by a call, a conversion or arithmetic is work.

## Rejected alternatives
- **Depending on stern4rust's crate.** The core may name only `quote` and `syn`
  ([ADR-TechnologyFreeCore](ADR-TechnologyFreeCore.md)); the shapes are small
  enough to hold here, pinned by tests of their own.
- **Counting a value trait's hand-written methods as a record's.** stern4rust
  lets a record implement `Display`, `FromStr` or `From`, but a hand-written
  `fmt` or `from_str` is code a test can pin, so a trait impl carrying a method
  keeps the file in scope here, as it always did.

## Consequences
A getter-only record, a record with steps, accessors spread over several
inherent impl blocks and a constructor of any name are all data holders; a `new`
whose fields are not bare parameters is not. An empty inherent impl exempts
nothing: at least one accessor must be present.

## Enforcement
`AccessorClassifier` holds the three shapes, `DataHolderDetector` applies them to
a file, and their tests pin every shape and every way out of it.

## Related
- [ADR-AMirrorMustBeEarned](ADR-AMirrorMustBeEarned.md)
- [ADR-StructuralExclusionsOverSemanticImportance](ADR-StructuralExclusionsOverSemanticImportance.md)
- `docs/RULES.md`, rule 5
