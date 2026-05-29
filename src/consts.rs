

/// Maximum stack size in words.
pub const MAX_STACK: usize = 256;

/// Hard limit on the number of instructions a program may execute.
pub const MAX_RUNTIME: usize = u16::MAX as usize;

pub type MachineWord = f32;

pub type Instruction = u8;

pub const INSTR_PER_WORD: usize = std::mem::size_of::<MachineWord>() / std::mem::size_of::<Instruction>();

