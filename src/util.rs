//! General utility functions

// Ok here since the `.try_into()` calls are guaranteed to work
// All sized are defined at complie time.
#![allow(clippy::missing_panics_doc)]

use crate::consts::{
    ADDRESS_SIZE, Address, INSTR_PER_ADDRESS, INSTR_PER_WORD, INSTRUCTION_SIZE, Instruction,
    MachineWord, WORD_SIZE,
};
use crate::instr::OpCode;

/// Turn a set of [`Instruction`]s into its representaion as an [`MachineWord`].
///
/// This representaion is the one used by the VM to accept immediate values.
pub fn word_from_instructions(parts: [Instruction; INSTR_PER_WORD]) -> MachineWord {
    let mut bytes = [0_u8; WORD_SIZE];
    for (part, chunk) in parts.iter().zip(bytes.chunks_mut(INSTRUCTION_SIZE)) {
        chunk.copy_from_slice(&part.to_le_bytes());
    }
    MachineWord::from_le_bytes(bytes)
}

/// Turn a [`MachineWord`] into its representaion as [`Instruction`]s.
///
/// Primarily used to encode immediate values into bytecode.
pub fn instructions_from_word(word: MachineWord) -> [Instruction; INSTR_PER_WORD] {
    let bytes = word.to_le_bytes();
    let mut parts = [Instruction::default(); INSTR_PER_WORD];
    for (part, chunk) in parts.iter_mut().zip(bytes.chunks(INSTRUCTION_SIZE)) {
        *part = Instruction::from_le_bytes(chunk.try_into().unwrap());
    }
    parts
}

/// Turn a set of [`Instruction`]s into an [`Address`] type.
///
/// Used by the VM to accept jump target addresses from the bytecode.
pub fn address_from_instructions(parts: [Instruction; INSTR_PER_ADDRESS]) -> Address {
    let mut bytes = [0_u8; ADDRESS_SIZE];
    for (part, chunk) in parts.iter().zip(bytes.chunks_mut(INSTRUCTION_SIZE)) {
        chunk.copy_from_slice(&part.to_le_bytes());
    }
    Address::from_le_bytes(bytes)
}

/// Turn a [`Address`] into its representaion as [`Instruction`]s.
///
/// Primarily used to encode jump target addresses into bytecode.
pub fn instructions_from_address(addr: Address) -> [Instruction; INSTR_PER_ADDRESS] {
    let bytes = addr.to_le_bytes();
    let mut parts = [Instruction::default(); INSTR_PER_ADDRESS];
    for (part, chunk) in parts.iter_mut().zip(bytes.chunks(INSTRUCTION_SIZE)) {
        *part = Instruction::from_le_bytes(chunk.try_into().unwrap());
    }
    parts
}

/// Append a jump [`OpCode`] with a placeholder address to `program`.
///
/// Returns the byte offset of the address slot so it can be filled in
/// later with [`patch_jmp`] once the target address is known.
pub fn push_jmp(program: &mut Vec<Instruction>, opcode: OpCode) -> usize {
    program.push(opcode.into());
    let offset = program.len();
    program.extend_from_slice(&instructions_from_address(0));
    offset
}

/// Overwrite the address slot at `offset` in `program` with `target`.
///
/// `offset` must be the value returned by a previous call to [`push_jmp`].
#[allow(clippy::cast_possible_truncation)]
pub fn patch_jmp(program: &mut [Instruction], offset: usize, target: usize) {
    program[offset..offset + INSTR_PER_ADDRESS]
        .copy_from_slice(&instructions_from_address(target as Address));
}
