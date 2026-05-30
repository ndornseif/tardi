#![doc = include_str!("../README.md")]

pub mod consts;
pub mod instr;
pub mod interpreter;
mod stack;
mod util;

// General TODOs:
// Add strings to asserts in tests.
// Finish and spell check comments.
// In the opcode enum allow cast from u8 to opcode respecting the modulo convention
//      Instead of deriving TryFromPrimitive.
// Add remaining instructions.
