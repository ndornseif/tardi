//! General utility functions.

use crate::consts::{MachineWord, MAX_STACK, Instruction, INSTR_PER_WORD};

pub fn word_from_instructions(parts: [Instruction; INSTR_PER_WORD]) -> MachineWord {
    unsafe { std::mem::transmute(parts) }
}


pub trait WrappingGet<T> {
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

