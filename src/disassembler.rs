//! Disassemble bytecode

use strum::EnumCount as _;

use crate::consts::{
    FormatImm as _, INSTR_PER_ADDRESS, INSTR_PER_WORD, INSTRUCTION_SIZE, Instruction,
};
use crate::instr::OpCode;
use crate::util::{address_from_instructions, word_from_instructions};

/// Write disassembly of `program` to `w`.
///
/// Each line shows: `address: hex_bytes  mnemonic  [value]`
///
/// Immediate [`MachineWord`](crate::consts::MachineWord) values are shown in scientific notation.
/// Jump targets are shown as zero-padded hex addresses.
/// Bytes extending past the end of the program are treated as zero.
///
/// ### Example
/// ```text
/// 0x000000: 02 00 00 a0 40 PushImm         5e0
/// 0x000005: 06             PopOut
/// 0x000006: 18 0d 00 00 00 Jmp             0x0000000d
/// 0x00000b: 07             Add
/// 0x00000c: 09             Mul
/// 0x00000d: 01             Halt
/// 0x00000e: 1f             Halt
/// ```
///
/// # Errors
/// Will bubble up errors from `write!` operations on `w`.
#[allow(clippy::missing_panics_doc)]
pub fn disassemble_program(
    w: &mut impl std::fmt::Write,
    program: &[Instruction],
) -> std::fmt::Result {
    const WORD_DIGITS: usize = INSTRUCTION_SIZE * 2;
    let mut itr = program.iter().enumerate();
    while let Some((i, &byte)) = itr.next() {
        #[allow(clippy::cast_possible_truncation)]
        let op = OpCode::try_from(byte % OpCode::COUNT as Instruction)
            .expect("modulo guarantees a valid opcode index");
        match op {
            OpCode::PushImm => {
                let mut imm_parts = [Instruction::default(); INSTR_PER_WORD];
                for part in &mut imm_parts {
                    *part = itr.next().map_or(0, |(_, &b)| b);
                }
                let imm = word_from_instructions(imm_parts);
                write!(w, "{i:#08x}: {byte:0WORD_DIGITS$x}")?;
                for p in &imm_parts {
                    write!(w, " {p:0WORD_DIGITS$x}")?;
                }
                writeln!(w, " {op:15} {}", imm.format_imm())?;
            }
            OpCode::Jmp
            | OpCode::JmpZero
            | OpCode::JmpAprxZero
            | OpCode::JmpPos
            | OpCode::JmpFin => {
                let mut addr_parts = [Instruction::default(); INSTR_PER_ADDRESS];
                for part in &mut addr_parts {
                    *part = itr.next().map_or(0, |(_, &b)| b);
                }
                let addr = address_from_instructions(addr_parts);
                write!(w, "{i:#08x}: {byte:0WORD_DIGITS$x}")?;
                for p in &addr_parts {
                    write!(w, " {p:0WORD_DIGITS$x}")?;
                }
                writeln!(w, " {op:15} {addr:#010x}")?;
            }
            _ => {
                writeln!(
                    w,
                    "{i:#08x}: {byte:0WORD_DIGITS$x}{:width$} {op:15}",
                    "",
                    width = INSTR_PER_WORD * (WORD_DIGITS + 1)
                )?;
            }
        }
    }
    Ok(())
}
