#![doc = include_str!("../README.md")]

pub mod consts;
pub mod disassembler;
pub mod instr;
pub mod interpreter;
pub(crate) mod numeric;
mod stack;
pub mod util;
