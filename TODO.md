# TODO

## High — correctness and completeness

- [ ] Decide on endianness and whether to use `bytemuck` in `util.rs` for the
      transmute-based converter functions.
- [ ] Fix tests that fail when `MachineWord` is an integer type: `floor`, `ceil`, etc.
- [ ] Implement `JmpTos` — the remaining jump instruction missing from the interpreter.
- [ ] Add tests for the unimplemented jump opcodes: `JmpAprxZero`, `JmpPos`, `JmpFin`, `JmpTos`.
- [ ] Replace `TryFromPrimitive` on `OpCode` with a custom cast that respects the modulo
      convention, so any `u8` can be converted to an `OpCode` without a `Result`.

## Medium — features

- [ ] Run the test suite with `MachineWord` set to both a float and an integer type to
      catch regressions when the type is changed.
- [ ] Add functionality to convert an arbitrary program into a canonical form
      (e.g. replace out-of-range bytes with their in-range equivalents).
- [ ] Write a more complex disassembler showcase example.

## Low — cleanup and polish

- [ ] Consolidate and condense the macros used for opcode handler definitions.
- [ ] Move `use` statements to the `#[cfg(test)]` module where they are only needed in tests.
- [ ] Fix scattered smaller TODOs remaining in the source.
- [ ] In doc comments, replace bare `` `val` `` with `[`val`]` where the type is in scope.
- [ ] Rewrite assert failure messages in a consistent tense and tone.
      Not needed where the test has a single assert and the test name already describes it.
- [ ] Set up fuzzing.
- [ ] Spell check doc-comments.
- [ ] Introduce examples in public API functions.
