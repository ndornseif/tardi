//! Disassemble bytecode

use strum::EnumCount as _;

use crate::consts::{FormatImm as _, INSTR_PER_WORD, Instruction};
use crate::instr::OpCode;
use crate::util::word_from_instructions;

/// Write disassembly of `program` to `w`.
///
/// Each line shows: `address: hex_bytes  mnemonic  [immediate]`
///
/// Bytes that extend past the end of the program are treated as zero.
pub fn disassemble_program(w: &mut impl std::fmt::Write, program: &[u8]) -> std::fmt::Result {
    let mut itr = program.iter().enumerate();
    while let Some((i, &byte)) = itr.next() {
        #[allow(clippy::cast_possible_truncation)]
        let op = OpCode::try_from(byte % OpCode::COUNT as u8)
            .expect("modulo guarantees a valid opcode index");
        match op {
            // TODO: decode jump target from the following immediate bytes.
            OpCode::PushImm => {
                let mut imm_parts = [Instruction::default(); INSTR_PER_WORD];
                for part in &mut imm_parts {
                    *part = itr.next().map_or(0, |(_, &b)| b);
                }
                let imm = word_from_instructions(imm_parts);
                write!(w, "{i:<#06x}: {byte:02x}")?;
                for p in &imm_parts {
                    write!(w, " {p:02x}")?;
                }
                writeln!(w, " {op:<15} {}", imm.format_imm())?;
            }
            _ => {
                write!(w, "{i:<#06x}: {byte:02x}")?;
                for _ in 0..INSTR_PER_WORD {
                    write!(w, "   ")?;
                }
                writeln!(w, " {op:<15}")?;
            }
        }
    }
    Ok(())
}
