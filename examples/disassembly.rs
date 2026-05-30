//! Prints the disassembly of an example program to stdout.
//!
//! Showcases:
//! - Regular instructions and PushImm with an immediate value.
//! - That multiple byte values can map to the same instruction via modulo.

use strum::EnumCount as _;
use tardi::consts::{Instruction, MachineWord};
use tardi::disassembler::disassemble_program;
use tardi::instr::OpCode;
use tardi::util::instructions_from_word;

fn main() -> std::fmt::Result {
    let mut program: Vec<Instruction> = vec![
        // Pops two input values, adds them, and writes the result to output.
        OpCode::PushIn.into(),
        OpCode::PushIn.into(),
        OpCode::Add.into(),
        OpCode::PopOut.into(),
        // A jump and some arithmetic (jump target not yet decoded).
        OpCode::JmpAprxZero.into(),
        OpCode::Mul.into(),
        OpCode::Sqrt.into(),
        // Two different byte values that both map to Sqrt via modulo.
        Instruction::from(OpCode::Sqrt) + OpCode::COUNT as Instruction,
        OpCode::Halt.into(),
        OpCode::PushImm.into(),
    ];
    // Add immediate value used by `PushImm`.
    program.extend_from_slice(&instructions_from_word(123 as MachineWord));
    let mut s = String::new();
    disassemble_program(&mut s, &program)?;
    println!("{}", s);
    Ok(())
}
