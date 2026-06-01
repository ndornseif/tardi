# TARDI VM
Tiny stack VM where every sequence of bytes is a valid program.  
The VM is meant to allow genetic algorithms to explore a solution space more efficiently. A finalized program should then be adapted to some other execution platform.

## Architecture
Note that all constants mentioned here can be found in the `consts` module.  
The `Interpreter` struct holds a fixed-size main stack, a fixed-size output stack, an input list, a program counter, and execution metadata.

## Bytecode
Instructions are encoded as elements of type `Instruction`, by default `u8` but can be set to `u16` using the `long-instruction` feature.
During decoding they are taken modulo the total number of instructions.
This means that every possible byte maps to some valid instruction.  

Immediate `MachineWord` values are encoded as `INSTR_PER_WORD` consecutive instruction slots immediately following the opcode.
Jump target addresses are encoded as `INSTR_PER_ADDRESS` consecutive instruction slots.
Bytes that extend past the end of the program are treated as zero.

## The Stack
The stack size is set by the `MAX_STACK` constant (default 256), the data type it contains by the `MachineWord` type.

**Overflow:** when the stack is full, the oldest (bottom) element is silently discarded so the new element can be written to the top.  
**Underflow:** popping an empty stack returns `MachineWord::default()` (zero for numeric types). Neither condition panics or errors.

There is a separate output stack capped at `MAX_OUTPUT` words (default 16).
It has the same overflow behaviour as the main stack, so only the most recently output values are retained.

## ~~Quirks~~ Notable properties

- **Every byte sequence is valid.** No input can cause a decode error or panic inside the interpreter.
- **Jumps are always in-bounds.** All jump target addresses are taken modulo the program length, making it impossible to jump outside the program.
- **Execution is bounded.** The interpreter stops after `MAX_INSTRUCTIONS` (65535 by default) instructions regardless of program content.
- **Opcode numbering is fragile by design.** Instructions are decoded by `value % COUNT`, so inserting or removing an opcode shifts all subsequent mappings. Programs built against one opcode set are not forward-compatible.
- **Integer mode silences float-only instructions.** When `MachineWord` is an integer type, `Round`, `Trunc`, `Ceil`, and `Floor` become no-ops. `JmpAprxZero` behaves like `JmpZero` and `JmpFin` behaves like `Jmp`.

## Instructions

| Opcode | Stack effect | Description |
|---|---|---|
| `Nop` | — | Does nothing. |
| `Halt` | — | Stop execution. |
| `PushImm` | `-> v` | Push the next `INSTR_PER_WORD` bytes as a `MachineWord` immediate. |
| `PushIn` | `-> v` | Pop from the input list and push onto the stack. No-op if input is empty. |
| `PushOne` | `-> 1` | Push the literal value 1. |
| `PushZero` | `-> 0` | Push `MachineWord::default()` (zero for numeric types). |
| `PopOut` | `a ->` | Pop the top value and push it onto the output stack. |
| `Add` | `a b -> a+b` | |
| `Sub` | `a b -> a-b` | Top value is subtracted from the second. |
| `Mul` | `a b -> a*b` | |
| `Div` | `a b -> a/b` | Second value divided by top. |
| `ModDiv` | `a b -> a%b` | Second value modulo top. |
| `Min` | `a b -> min(a,b)` | |
| `Max` | `a b -> max(a,b)` | |
| `Sqrt` | `a -> sqrt(\|a\|)` | Integer mode uses `isqrt`. Note the use of abs.|
| `Round` | `a -> round(a)` | No-op in integer mode. |
| `Trunc` | `a -> trunc(a)` | No-op in integer mode. |
| `Ceil` | `a -> ceil(a)` | No-op in integer mode. |
| `Floor` | `a -> floor(a)` | No-op in integer mode. |
| `Neg` | `a -> -a` | |
| `Abs` | `a -> \|a\|` | |
| `Swap` | `a b -> b a` | Swap the top two elements. |
| `Remove` | `a ->` | Discard the top element. |
| `Dup` | `a -> a a` | Duplicate the top element. |
| `Jmp` | — | Jump to the next `INSTR_PER_ADDRESS` bytes as an address. |
| `JmpZero` | — | Jump if TOS equals zero (does not pop). |
| `JmpAprxZero` | — | Jump if TOS is within `10 * EPSILON` of zero (does not pop). Identical to `JmpZero` in integer mode. |
| `JmpPos` | — | Jump if TOS is positive (does not pop). |
| `JmpFin` | — | Jump if TOS is finite (does not pop). Always jumps in integer mode. |
| `JmpTos` | `a ->` | Pop TOS, take its absolute value, and jump to that address. |

All jump destinations are taken modulo program length.

## Building programs
The `util` module provides helpers for constructing bytecode in Rust:

- `instructions_from_word(w)` / `word_from_instructions(parts)` — convert between a `MachineWord` and its raw `Instruction` bytes.
- `instructions_from_address(a)` / `address_from_instructions(parts)` — same for jump addresses.
- `push_jmp(program, opcode)` — append a jump opcode with a zeroed placeholder address; returns the byte offset of the placeholder.
- `patch_jmp(program, offset, target)` — fill in the placeholder written by `push_jmp` once the target address is known.

These helpers are used throughout the test suite and examples.

## The disassembler
`disassembler::disassemble_program(w, program)` writes a human-readable listing to any `fmt::Write` target.

Each line follows the format `address: hex_bytes  mnemonic  [operand]`:
- Immediate values are shown in scientific notation for float `MachineWord` types and as plain integers for integer types.
- Jump targets are shown as zero-padded hex addresses. If the raw encoded address falls outside the program, the actual effective address after modulo reduction is shown as `raw -> effective`.

```text
0x0000: 03             PushIn         
0x0001: 1b 0f 00       JmpPos          0x000f
0x0004: 1a 09 00       JmpAprxZero     0x0009
0x0007: 16             Remove         
0x0008: 01             Halt           
0x0009: 16             Remove         
0x000a: 05             PushZero       
0x000b: 06             PopOut         
0x000c: 18 00 00       Jmp             0x0000
0x000f: 0e             Sqrt           
0x0010: 02 00 00 c8 42 PushImm         1e2
0x0015: 09             Mul            
0x0016: 0f             Round          
0x0017: 06             PopOut         
0x0018: 18 1b 00       Jmp             0x001b -> 0x0000
```

A disassembler example in `examples/disassembler.rs` can be used to disassemble programs from file or stdin. To view a fuzzer artifact for example:
```sh

cargo run --example disassemble --features int-word \
     fuzz/artifacts/fuzz_target_i32/crash-349d5a6c6ec6050d0e6651ebdb2dc2f8627e5aea
```
```
```

## Feature flags
Four independent feature flags control the concrete types used throughout the crate.
Changing a flag changes the opcode encoding of existing programs.

| Flag | Effect |
|---|---|
| _(none)_ | `MachineWord = f32`, `Instruction = u8`, `Address = u16` |
| `int-word` | `MachineWord` becomes the signed integer variant (`i32` or `i64`). Float-only instructions become no-ops. |
| `long-word` | `MachineWord` doubles in width (`f64` or `i64`). `INSTR_PER_WORD` doubles accordingly. |
| `long-instruction` | `Instruction` becomes `u16`. Doubles the opcode space and the width of all embedded immediates and addresses. |
| `long-address` | `Address` becomes `u32`. Allows programs up to ~4 GiB instead of 64 KiB. Note that utilizing this requires raising `MAX_INSTRUCTIONS` or setting a custom instruction limit with `Interpreter::new_with_program_and_limit()`. |

## Performance
- `MAX_STACK` and `MAX_OUTPUT` should be powers of two. The ring-buffer index is computed as `push_count % N`; a power of two allows the compiler to replace this with a bitwise AND.
- Execution is capped at `MAX_INSTRUCTIONS` = 65535 by default, bounding the worst-case runtime of any program.

## just
This project uses [just](https://github.com/casey/just) to manage testing and fuzzing with different sets of crate features enabled. Use `just test-all` to test all relevant crate feature combinations. Use `just lint` to run Clippy. Use `just examples` to run the included examples.

## Fuzzing
Two fuzz targets exercise the interpreter with arbitrary bytecode and input data.

| Target | Feature | `MachineWord` |
|---|---|---|
| `fuzz_target_f32` | _(default)_ | `f32` |
| `fuzz_target_i32` | `int-word` | `i32` |

Run with `just fuzz-f32` or `just fuzz-i32`.

`tests/fuzzer_hits.rs` contains tests for all problems that were found by the fuzzer.

## License
This crate may be licensed under the [GNU Lesser General Public License, version 2.1](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html#SEC1).
