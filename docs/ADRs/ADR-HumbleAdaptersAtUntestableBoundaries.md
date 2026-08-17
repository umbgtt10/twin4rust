# ADR — Humble adapters at untestable boundaries

## Status

Accepted.

## Context

Some code exists only to reach something a unit test cannot reach: a probe
flashing a board, a system clock, a random source, a filesystem. The file that
performs that reach can never have a meaningful mirrored test, because the thing
it does is the thing a test would have to fake.

The usual answer is to invert the dependency — put a trait at the boundary, let
the caller take it, and test the caller against a fake. That answer is right
often enough to be a reflex, and the reflex is what this ADR is about.

A seam does not remove untestable code. It **concentrates** it. Before the seam
one file holds N lines of logic entangled with one boundary call, and all N are
out of reach. After it, the logic is testable and what remains untestable is a
one-line adapter. The trade is worth making when N is large. When N is zero —
when the file's whole body is the boundary call — the seam relocates the
untestable line, and charges a trait, a dynamic dispatch and a fake for doing so.

`BoardHalter` in `etheram-tools/embedded-gate` is the zero case:

```rust
pub fn halt(self) {
    BoardEraser::new(self.board_id, &self.probe_serial).erase();
}
```

There is no logic to move. A test written against a faked eraser would assert
that a one-line delegation delegates — restating the code rather than
constraining it. Yet the gate demanded that test, because the file declares a
type with a method and so matches none of the existing exclusions.

## Decision

A file is exempt when it declares a single type whose inherent methods are all
either a trivial `new` or a **forwarding method**, where forwarding means:

- the method returns nothing — no return type, or `-> ()`
- its body is exactly one expression statement
- that expression is a call or a method call

Nothing else. Not a macro, not an operator, not a literal, not an `if`, `match`,
block or loop.

The return type carries most of the weight. A method that returns a value
*produces* something, and what it produces is worth asserting —
`to_label(&self) -> String` composes and stays in scope. A method that returns
nothing *delegates an effect*, and where the effect lands is the only thing to
observe. That distinction is cheap to compute and hard to get wrong, which is
why it is preferred over inspecting argument shapes.

## Forcing constraints / Evidence

This is the same insight the trivial-constructor rule already encodes, drawn one
method further along. That rule exempts a type that only **holds** what it was
given. This one also exempts a type that **forwards** what it was given. Neither
has anything a mirrored test could pin that reading the file would not.

The rule is self-policing, which is the property that makes it safe. An adapter
is exempt precisely while it stays humble. Add a branch, a retry, an error
interpretation or a unit conversion, and it stops being a single call statement,
the exemption lapses, and the gate asks for a test again — at exactly the moment
the file starts deserving one. The exclusion is not "stop looking at this file";
it is "look at this file when it grows a decision".

Correctness of the adapters themselves comes from a different tier and should
not be claimed by this one. `embedded-gate` runs firmware under QEMU and on real
boards; those runs exercise the adapters continuously. A gate that demanded a
`*_tests.rs` beside a probe-rs invocation would be asserting something outside
its competence.

## Rejected alternatives

- **An opt-out attribute or ignore list.** The obvious escape hatch, and it
  contradicts `ADR-StructuralExclusionsOverSemanticImportance` directly: the tool
  does not accept semantic judgments about importance, and an annotation is
  nothing else. Once it exists it becomes the thing sprinkled to silence the
  gate, and the gate starts measuring diligence rather than structure.
- **Requiring the seam anyway.** Produces tautology tests — a fake asserting that
  a delegation delegates — and buys a trait, a dyn dispatch and a mock per
  boundary. It also does not help: the adapter behind the new seam is itself a
  file with no mirror, so the gap moves rather than closing.
- **Inspecting argument shapes** to tell forwarding from computing. Strictly more
  precise, considerably more machinery, and the return-type test already
  separates the cases that matter.
- **A package-name convention**, as `-validation` uses. A reasonable answer when
  adapters cluster into their own crate, and still available. It is the wrong
  granularity when adapters sit among ordinary code, as in
  `embedded-gate/src/hardware/`.

## Consequences

A forwarding method that forwards to the wrong place is now excluded, and the
gate will not catch it. That is the same trade the trivial-constructor rule
already accepts: a `new` that assigns the wrong field is equally invisible. The
mitigation is the humbleness requirement itself — a single call statement is
short enough that reading it is the verification.

Files whose methods all return values remain in scope however thin they are, so
delegating getters are still reported. Ambiguity resolves toward visible, as
elsewhere in this tool.

The rule reads the statement, not the arguments, so computation passed into a
forwarded call — `sink::send(self.left + self.right)` — is exempt. Two things
bound that leak: only one statement is permitted, and any helper doing the
computing would return a value and take the file back out of the exemption. The
alternative, walking argument expressions for operators, buys precision at a
cost the return-type test already mostly covers.

## Enforcement

`HumbleAdapterDetector` takes parsed `syn` items and no resolver, manifest or
filesystem. `tests/humble_adapter_detector_tests.rs` pins the rule and its
boundaries: the `BoardHalter` shape is exempt, a computing method with the same
one-line body is not, a method returning a value is not, and a body carrying any
branch is not.
