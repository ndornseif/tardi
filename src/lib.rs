#![doc = include_str!("../README.md")]

pub mod consts;
pub mod disassembler;
pub mod instr;
pub mod interpreter;
mod stack;
pub mod util;

// General TODOs:
// In the opcode enum allow cast from u8 to opcode respecting the modulo convention
//      Instead of deriving TryFromPrimitive.
// Add remaining jmp tos instruction.
// functionality to turn arbitrary programs into a canonical version.
// Consolodate and condense macros for opcode and function defs
// In doccomments swap `val` for [`val`] where appropriate
// Fix scattered smaller TODOs
// Decide on endianess, bytemuck usage ect. in util converter functions.
// Fix failing tests when using ints as MachineWord: floor, ceil, ect.
// Autorun tests with MachineWord as int and float?
// test: missing jmps 
// Check if use statements can be moved from main body to test moule if
//      not needed in main.
