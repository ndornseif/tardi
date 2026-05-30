//! Disassemble bytecode

use strum::EnumCount as _;

use crate::consts::{FormatImm as _, INSTR_PER_WORD, INSTRUCTION_SIZE, Instruction};
use crate::instr::OpCode;
use crate::util::word_from_instructions;

/// Write disassembly of `program` to `w`.
///
/// Each line shows: `address: hex_bytes  mnemonic  [immediate]`
///
/// ### Example:  
/// ```text
/// 0x0000: 03             PushIn         
/// 0x0001: 03             PushIn         
/// 0x0002: 07             Add            
/// 0x0003: 06             PopOut         
/// 0x0004: 02 00 94 bf 44 PushImm         1.532625e3
/// 0x0009: 18             JmpAprxZero    
/// 0x000a: 09             Mul            
/// 0x000b: 0e             Sqrt           
/// 0x000c: 29             Sqrt           
/// 0x000d: 01             Halt           
/// ```
/// When an immediate value would extend beyond the end
/// of the program the missing values are assumed as zero.
///
/// # Errors
/// Will bubble up errors from `write!` operations on `w`.
#[allow(clippy::missing_panics_doc)]
pub fn disassemble_program(
    w: &mut impl std::fmt::Write,
    program: &[Instruction],
) -> std::fmt::Result {
    const WORD_DIGITS: usize = INSTRUCTION_SIZE * 2; // Number of hex digits
    let mut itr = program.iter().enumerate();
    while let Some((i, &byte)) = itr.next() {
        #[allow(clippy::cast_possible_truncation)]
        let op = OpCode::try_from(byte % OpCode::COUNT as Instruction)
            .expect("modulo guarantees a valid opcode index");
        match op {
            // TODO: decode jump target from the following immediate bytes.
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
