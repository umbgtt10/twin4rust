# Architecture Decision Records

Each ADR documents one load-bearing decision behind `cargo-twin4rust` —
succinct, self-contained, citable on its own. Like the sibling `crap4rust`
tool and unlike the larger `etheram` ecosystem repos, these are not
priority-tiered: a single-crate CLI has a small enough decision surface that a
flat list is sufficient.

## Index

| ADR | Decision |
|---|---|
| [ADR-MirroredPathIsTheWholeContract](ADR-MirroredPathIsTheWholeContract.md) | The contract is a path, not a quality judgement — `twin4rust` checks that `tests/<path>/<name>_tests.rs` exists and never opens it, so the tool has no opinion it could be wrong about. |
| [ADR-NoCrossPackageSatisfaction](ADR-NoCrossPackageSatisfaction.md) | No flag lets one package's tests satisfy another package's mirrored-file expectation — such a flag shipped briefly and was removed after it improved the number while coverage stayed flat. |
| [ADR-StructuralExclusionsOverSemanticImportance](ADR-StructuralExclusionsOverSemanticImportance.md) | Exclusions are decided from the `syn` AST by structure alone, never by inferring how important a file is — ambiguous files stay visible rather than being silently dropped. |
| [ADR-CargoSubcommandPackaging](ADR-CargoSubcommandPackaging.md) | The crate publishes as `cargo-twin4rust` with library `twin4rust`, and strips the subcommand name cargo re-inserts at `argv[1]`. |

## Template

```markdown
# ADR-<Name>

- **Status:** Accepted | Proposed | Superseded by <ADR>
- **Date:** YYYY-MM-DD

## Context
The forces and tension this resolves.

## Decision
The choice, in one quotable sentence.

## Forcing constraints / Evidence
Why this was forced, not freely chosen — the real evidence. `N/A` if none.

## Rejected alternatives
What we did not do, and why.

## Consequences
What it commits us to; what it costs; obligations pushed onto consumers.

## Enforcement
The specific test, gate, or structural mechanism that keeps it true.
`N/A` if purely structural.

## Related
Links to other ADRs and architecture docs.
```

Fields that do not apply are marked `N/A` rather than padded. Each ADR is a
snapshot of the decision as it stands today, not a changelog — state the
current shape as fact, don't narrate what an earlier version of this document
used to say.
