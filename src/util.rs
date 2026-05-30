//! General utility functions

use crate::consts::{INSTR_PER_WORD, Instruction, MachineWord};

pub fn word_from_instructions(parts: [Instruction; INSTR_PER_WORD]) -> MachineWord {
    // SAFETY:
    // `INSTR_PER_WORD` is calculated based on `mem::size_of` to ensure correct size.
    // That all sequences of bytes must represent a valid value
    // is a fundamental restriction placed on `MachineWord` and `Instruction` by design.
    // These are IEEE floating point or integer values.
    unsafe { std::mem::transmute(parts) }
}

/// Enables a wrapping version of `.get()` that
/// selects the element as: `slice[index % slice.len()]`.
pub trait WrappingGet<T> {
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

