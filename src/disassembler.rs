//! Disassemble bytecode

use crate::consts::{
    ADDRESS_SIZE, Address, INSTR_PER_ADDRESS, INSTR_PER_WORD, INSTRUCTION_SIZE, Instruction,
};
use crate::instr::OpCode;
use crate::numeric::FormatImm as _;
use crate::util::{address_from_instructions, word_from_instructions};

/// Write disassembly of `program` to `w`.
///
/// Each line shows: `address: hex_bytes  mnemonic  [value]`
///
/// Immediate [`MachineWord`](crate::consts::MachineWord) values are shown in scientific notation
/// when they are floats.
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
    const COL_WIDTH: usize = WORD_DIGITS + 1; // " XX" per byte column
    // Number of hex digits to encode address, plus to to acoount for 0x prefix.
    const ADDRESS_DIGITS: usize = ADDRESS_SIZE * 2 + 2;
    let max_arg_bytes = INSTR_PER_WORD.max(INSTR_PER_ADDRESS);

    let mut itr = program.iter().enumerate();
    while let Some((i, &byte)) = itr.next() {
        let op = OpCode::from(byte);
        match op {
            OpCode::PushImm => {
                let mut imm_parts = [Instruction::default(); INSTR_PER_WORD];
                for part in &mut imm_parts {
                    *part = itr.next().map_or(0, |(_, &b)| b);
                }
                let imm = word_from_instructions(imm_parts);
                write!(w, "{i:#0ADDRESS_DIGITS$x}: {byte:0WORD_DIGITS$x}")?;
                for p in &imm_parts {
                    write!(w, " {p:0WORD_DIGITS$x}")?;
                }
                let pad = (max_arg_bytes - INSTR_PER_WORD) * COL_WIDTH;
                writeln!(w, "{:pad$} {op:15} {}", "", imm.format_imm())?;
            }
            OpCode::Jmp
            | OpCode::JmpZero
            | OpCode::JmpAprxZero
            | OpCode::JmpPos
            | OpCode::JmpFin => {
                let p_len = program.len() as Address;
                let mut addr_parts = [Instruction::default(); INSTR_PER_ADDRESS];
                for part in &mut addr_parts {
                    *part = itr.next().map_or(0, |(_, &b)| b);
                }
                let addr = address_from_instructions(addr_parts);
                write!(w, "{i:#0ADDRESS_DIGITS$x}: {byte:0WORD_DIGITS$x}")?;
                for p in &addr_parts {
                    write!(w, " {p:0WORD_DIGITS$x}")?;
                }
                let pad = (max_arg_bytes - INSTR_PER_ADDRESS) * COL_WIDTH;
                if addr >= p_len {
                    let mod_addr = addr % p_len;
                    writeln!(
                        w,
                        "{:pad$} {op:15} {addr:#0ADDRESS_DIGITS$x} -> {mod_addr:#0ADDRESS_DIGITS$x}",
                        ""
                    )?;
                } else {
                    writeln!(w, "{:pad$} {op:15} {addr:#0ADDRESS_DIGITS$x}", "")?;
                }
            }
            _ => {
                let pad = max_arg_bytes * COL_WIDTH;
                writeln!(
                    w,
                    "{i:#0ADDRESS_DIGITS$x}: {byte:0WORD_DIGITS$x}{:pad$} {op:15}",
                    ""
                )?;
            }
        }
    }
    Ok(())
}
