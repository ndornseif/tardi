# TODO

## High

- [ ] Decide on endianness and whether to use `bytemuck` in `util.rs` for the
      transmute-based converter functions.
- [x] Fix tests that fail when `MachineWord` is an integer type: `floor`, `ceil`, etc.
- [x] Implement `JmpTos` — the remaining jump instruction missing from the interpreter.
- [x] Add tests for the unimplemented jump opcodes: `JmpAprxZero`, `JmpPos`, `JmpFin`, `JmpTos`.
- [ ] Replace `TryFromPrimitive` on `OpCode` with a custom cast that respects the modulo
      convention, so any `u8` can be converted to an `OpCode` without a `Result`.

## Medium

- [x] Consolidate jump implementations using macros.
- [ ] Add functionality to convert an arbitrary program into a canonical form
      (e.g. replace out-of-range bytes with their in-range equivalents).
- [ ] Write a more complex disassembler showcase example.
- [ ] Decide on feature flags for using other types as `MachineWord`.
      Desirable: `i8`, `i16`, `i32`, `i64` and `f64`.
      Potentially have a `int` and `float` flag that set `MachineWord` to `i32` and `f32` 
      by default. Write code in a way that allows `i8` - `i64` with one flag 
      and `f32`, `f64` with other flag.
- [ ] After feature flags are finalized optimize code using `#[cfg]`.
      Lots of integer operations can be simplified and speed up.
      The use of traits can be reduced. Almost all code in `numeric.rs` removed.

## Low

- [ ] Move `use` statements to the `#[cfg(test)]` module where they are only needed in tests.
- [ ] Fix scattered smaller TODOs remaining in the source.
- [ ] In doc comments, replace bare `` `val` `` with `[`val`]` where the type is in scope.
- [ ] Rewrite assert failure messages in a consistent tense and tone.
      Not needed where the test has a single assert and the test name already describes it.
- [ ] Fuzz the `i32` `MachineWord` crate feature.
- [ ] Spell check doc-comments.
- [ ] Introduce examples in public API functions.
