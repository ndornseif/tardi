//! General utility functions

use crate::consts::{Address, INSTR_PER_ADDRESS, INSTR_PER_WORD, Instruction, MachineWord};
use crate::instr::OpCode;

/// Turn a set of [`Instruction`]s into its representaion as an [`MachineWord`].
///
/// This representaion is the one used by the VM to accept immediate values.
pub fn word_from_instructions(parts: [Instruction; INSTR_PER_WORD]) -> MachineWord {
    // SAFETY:
    // `INSTR_PER_WORD` is calculated based on `mem::size_of` to ensure correct size.
    // That all sequences of bytes must represent a valid value
    // is a fundamental restriction placed on [`MachineWord`] and [`Instruction`] by design.
    // These are IEEE floating point or integer values.
    // Outstanding decision on endianess.
    unsafe { std::mem::transmute(parts) }
}

/// Turn a set of [`Instruction`]s into an [`Address`] type.
///
/// Used by the VM to accept jump target addresses from the bytecode.
pub fn address_from_instructions(parts: [Instruction; INSTR_PER_ADDRESS]) -> Address {
    // SAFETY:
    // See [`word_from_instructions`].
    unsafe { std::mem::transmute(parts) }
}

/// Turn a [`MachineWord`] into its representaion as [`Instruction`]s.
///
/// Primarily used to encode immediate values into bytecode.
#[allow(clippy::transmute_num_to_bytes)]
pub fn instructions_from_word(word: MachineWord) -> [Instruction; INSTR_PER_WORD] {
    // SAFETY:
    // See [`word_from_instructions`].
    unsafe { std::mem::transmute(word) }
}

/// Turn a [`Address`] into its representaion as [`Instruction`]s.
///
/// Primarily used to encode jump target addresses into bytecode.
#[allow(clippy::transmute_num_to_bytes)]
pub fn instructions_from_address(addr: Address) -> [Instruction; INSTR_PER_ADDRESS] {
    // SAFETY:
    // See: [`word_from_instruction`].
    unsafe { std::mem::transmute(addr) }
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
