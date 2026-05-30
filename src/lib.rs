#![doc = include_str!("../README.md")]

pub mod consts;
pub mod instr;
pub mod interpreter;
mod stack;
mod util;

// General TODOs:
// Add strings to asserts in tests.
// Adapt stack to arbitrary const size and arbitrary type.
// Use mw! macro.
// Finish ans spell check comments.
// In the opcode enum allow cast from u8 to opcode respecting the modulo convention
//      Instead of deriving TryFromPrimitive.
