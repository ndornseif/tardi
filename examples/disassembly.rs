//! Prints the disassembly of an example program to stdout.
//!
//! Demonstrates:
//! - Regular instructions and PushImm with an immediate value.
//! - That multiple byte values can map to the same instruction via modulo.

use strum::EnumCount as _;
use tardi::disassembler::disassemble_program;
use tardi::instr::OpCode;
use tardi::consts::Instruction;

fn main() -> std::fmt::Result {
    let program: Vec<Instruction> = vec![
        // Pops two input values, adds them, and writes the result to output.
        OpCode::PushIn.into(),
        OpCode::PushIn.into(),
        OpCode::Add.into(),
        OpCode::PopOut.into(),
        // Push the float immediate 1532.56 onto the stack.
        OpCode::PushImm.into(),
        0x00, 0x94, 0xbf, 0x44,
        // A jump and some arithmetic (jump target not yet decoded).
        OpCode::JmpAprxZero.into(),
        OpCode::Mul.into(),
        OpCode::Sqrt.into(),
        // Two different byte values that both map to Sqrt via modulo.
        Instruction::from(OpCode::Sqrt) + OpCode::COUNT as Instruction,
        OpCode::Halt.into(),
    ];
    let mut s = String::new();
    disassemble_program(&mut s, &program)?;
    println!("{}", s);
    Ok(())
}
