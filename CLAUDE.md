# twin4rust

## Meaning

`twin4rust` is a cargo subcommand that reports Rust source files with no
mirrored test file beside them — `src/<path>/<name>.rs` expects
`tests/<path>/<name>_tests.rs`.

It is not a coverage tool. It measures no lines and no branches, and it has no
opinion on whether existing tests are good. It answers one structural question:
does the mirrored test file exist?

`docs/RULES.md` is the canonical policy description. If the mirrored-test
policy changes, update `docs/RULES.md`, the tool logic and the integration
tests together. `docs/ARCHITECTURE.md` maps the code; `docs/ADRs/` holds the
load-bearing decisions.

It is self-contained.

## Boundary Rule

This repository is **SELF-CONTAINED**.

The LLM **SHALL NOT cross its boundaries without asking**.

That means:
- do not inspect, edit, or rely on files outside `twin4rust/` unless the user explicitly asks
- do not pull assumptions from sibling repositories or crates
- do not propose cross-repository changes by default

## Quality Gates

### Mandatory after every change to `src/` or `tests/`

Run gates:

`powershell -File scripts\run_stage_1.ps1`
`powershell -File scripts\run_stage_2.ps1`

If either gate is not green, the work is not complete.

Stage 1 is formatting, clippy and tests -- cargo built-ins only, so it works on
a fresh checkout. Stage 2 is four gates, run in this order:

| gate | asks |
|---|---|
| `cargo stern4rust` | do the house coding rules hold |
| `cargo crap4rust` | is any function complex and untested |
| `cargo twin4rust` | does every source file have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk too high |

stern4rust runs **first** because its corrections are renames, file moves and
directory splits: a layout it is about to reject is a layout the others would
have measured for nothing. Its findings are also the cheapest to act on.

All twenty-one of its rules are enforced, with nothing skipped and nothing
unconfigured. `docs/header.txt` holds the three-line header every `.rs` file
carries and `stern4rust.toml` names it -- in the config rather than the gate
script, so a hand-run of `cargo stern4rust` checks exactly what the gate checks.

`cargo install cargo-stern4rust`
`cargo install cargo-crap4rust`
`cargo install cargo-iceberg4rust`

The twin4rust gate runs this tool against itself. A tool that enforces a rule it
does not satisfy is not worth installing, so every behaviour-bearing source file
here keeps its own mirrored `<name>_tests.rs`.

## Publishing

The crate publishes as `cargo-twin4rust` so cargo resolves `cargo twin4rust`,
matching `cargo-crap4rust` and `cargo-dry4rust`. The library is `twin4rust`.

`src/main.rs` strips the repeated subcommand name that cargo inserts as
`argv[1]`; running the binary directly does not repeat it, so the strip is
conditional. `Args::without_cargo_subcommand` owns that rule and is tested.

Before publishing, `cargo publish --dry-run` must succeed.

## Orthogonality, trait surface and cognitive complexity

**When changing productive code, always maximize orthogonality and testable surface through traits, and minimize cognitive complexity.**

Specifically:
- prefer extracting behavior behind traits so individual pieces can be tested and swapped independently
- prefer small, focused methods with a single responsibility over large methods with many branches
- prefer named structs with methods over free functions operating on external state
- when `crap4rust` or a reviewer flags a function as too complex, reduce it by extracting internal structs with methods and adding integration coverage — not by extracting standalone helper functions
- never increase cognitive complexity to pass a test; find the root cause and fix it there
- make constructors depend on traits, not directly on concrete implementations
- ALL dependencies are injected through the SINGLE constructor and stored in the struct
- apply the same split recursively to nested dependencies: trait first, state/data model second, concrete implementation third

## User coding standards

- one struct per file
- no unnecessary comments in code
- unit tests are not allowed. Only integration tests are
- consolidate scattered functions inside structs as appropriate
- no `&mut` input parameters; prefer return values
- only use `pub mod` in `mod.rs` and `lib.rs`
- split test files so there is one test file per source file, named `<source file name>_tests.rs`
- in `all_tests.rs`, reference test files one by one without `#[path = ...]`
- apply AAA (`Arrange`, `Act`, `Assert`) structure to tests with blank-line separation between the three sections
- use `// Arrange & Act` if there is no separate `Arrange`
- use `// Act & Assert` if there is no separate `Act`
- add the repository copyright and license header to every Rust source file
- tests should be named as follows `<method under test>_<test description>_<result>`
- do not use fully qualified paths; use `use` imports instead
