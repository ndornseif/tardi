#![doc = include_str!("../README.md")]

pub mod consts;
pub mod instr;
pub mod interpreter;
pub mod disassembler;
mod stack;
mod util;

// General TODOs:
// In the opcode enum allow cast from u8 to opcode respecting the modulo convention
//      Instead of deriving TryFromPrimitive.
// Add remaining instructions.
// functionality to turn arbitrary programs into a canonical version.
// change handling of immediate values when they extend past the end of the program.
//      Instead of current wrapping behaviour assume zero for missing bytes?
// Allow printing instruction vec in nice human readable format
//      Handle immediate and jump targets.
// Store reason for halting? Allow caller to tell when MAX_RUNTIME exceeded.
// Consolodate and condense macros for opcode and function defs
// In doccomments swap `val` for [`val`] where appropriate
