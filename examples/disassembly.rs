//! Prints the disassembly of an example program to stdout.
//!
//! Showcases:
//! - `PushImm` with a decoded float immediate.
//! - `Jmp` with its decoded target address.
//! - Dead code that is jumped over.
//! - Two byte values that map to the same instruction via modulo.

use strum::EnumCount as _;
use tardi::consts::{Address, INSTR_PER_ADDRESS, Instruction};
use tardi::disassembler::disassemble_program;
use tardi::instr::OpCode;
use tardi::mw;
use tardi::util::{instructions_from_address, instructions_from_word};

fn main() -> std::fmt::Result {
    let mut program: Vec<Instruction> = Vec::new();

    // Push 5.0 and send it to the output.
    program.push(OpCode::PushImm.into());
    program.extend_from_slice(&instructions_from_word(mw!(5)));
    program.push(OpCode::PopOut.into());

    // Jump past dead code to Halt.
    program.push(OpCode::Jmp.into());
    let jump_addr_offset = program.len();
    program.extend_from_slice(&instructions_from_address(0)); // patched below

    // Dead code: never reached because of the Jmp above.
    program.push(OpCode::Add.into());
    program.push(OpCode::Mul.into());

    // Patch the Jmp target to point at Halt.
    let halt_pos = program.len() as Address;
    program[jump_addr_offset..jump_addr_offset + INSTR_PER_ADDRESS]
        .copy_from_slice(&instructions_from_address(halt_pos));

    program.push(OpCode::Halt.into());

    // Byte 0x1f = 31 also maps to Halt (31 % COUNT = 1 = Halt).
    program.push(Instruction::from(OpCode::Halt) + OpCode::COUNT as Instruction);

    let mut s = String::new();
    disassemble_program(&mut s, &program)?;
    println!("{}", s);
    Ok(())
}
