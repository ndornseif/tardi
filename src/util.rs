//! General utility functions

use crate::consts::{Address, INSTR_PER_ADDRESS, INSTR_PER_WORD, Instruction, MachineWord};

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

/// Turn a set of [`Instruction`]s into an [`AdDress`] type.
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
pub fn instructions_from_word(word: MachineWord) -> [Instruction; INSTR_PER_WORD] {
    // SAFETY:
    // See [`word_from_instructions`].
    unsafe { std::mem::transmute(word) }
}

/// Turn a [`Address`] into its representaion as [`Instruction`]s.
///
/// Primarily used to encode jump target addresses into bytecode.
pub fn instructions_from_address(addr: Address) -> [Instruction; INSTR_PER_ADDRESS] {
    // SAFETY:
    // See: [`word_from_instruction`].
    unsafe { std::mem::transmute(addr) }
}

/// Enables a wrapping version of `.get()` that
/// selects the element as: `slice[index % slice.len()]`.
pub(crate) trait WrappingGet<T> {
    /// Returns the element at position `index % len`.
    /// If len is zero `T::default()` is returned.
    fn wrapping_get(&self, index: usize) -> T;
}

impl<T: Default + Clone> WrappingGet<T> for [T] {
    fn wrapping_get(&self, index: usize) -> T {
        if self.is_empty() {
            T::default()
        } else {
            self[index % self.len()].clone()
        }
    }
}
