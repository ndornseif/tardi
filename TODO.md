# TODO

## High

- [x] Decide on endianness and whether to use `bytemuck` in `util.rs` for the
      transmute-based converter functions.
- [x] Fix tests that fail when `MachineWord` is an integer type: `floor`, `ceil`, etc.
- [x] Implement `JmpTos` — the remaining jump instruction missing from the interpreter.
- [x] Add tests for the unimplemented jump opcodes: `JmpAprxZero`, `JmpPos`, `JmpFin`, `JmpTos`.
- [x] Replace `TryFromPrimitive` on `OpCode` with a custom cast that respects the modulo
      convention, so any `u8` can be converted to an `OpCode` without a `Result`.

## Medium

- [x] Get README into a usable state.
- [x] Consolidate jump implementations using macros.
- [ ] Add functionality to convert an arbitrary program into a canonical form
      (e.g. replace out-of-range bytes with their in-range equivalents).
- [x] Write a more complex disassembler showcase example.
- [x] Clean up dead code in numeric.rs now that int/float switch flag exists.
- [ ] Implement additional ways to process interpreter output than `.output()`?
- [ ] Reverse direction of output when returned from interpreter?
- [x] Update disassembler to output addresses with the number of hex digits dictated by 
      their bit width.
- [ ] Make use of feature flags in disassembler to replace `format_imm`. Remove it from 
      numeric.rs
- [x] Allow changing the execution limit at runtime.
- [x] Expand disassembler to deal with fuzzer crash artifacts that contain data not just
      instructions.
- [x] Add example to print execution of program step by step for debugging.
- [x] More ergonomic API for `Interpreter`, any new methods that would make sense?
- [ ] Explore what happens when one jumps into the middle of an imm value or jmp target address.
- [ ] Update README with examples.

## Low

- [ ] Move `use` statements to the `#[cfg(test)]` module where they are only needed in tests.
- [ ] Fix scattered smaller TODOs remaining in the source.
- [ ] In doc comments, replace bare `` `val` `` with `[`val`]` where the type is in scope.
- [ ] Rewrite assert failure messages in a consistent tense and tone.
      Not needed where the test has a single assert and the test name already describes it.
- [x] Fuzz the `i32` `MachineWord` crate feature.
- [ ] Spell check doc-comments.
- [ ] Introduce examples in public API functions.
- [x] Make interpreter cut program short if it is longer than `Address::MAX`.
