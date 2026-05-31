//! Project wide constant definitions

/// Maximum stack size in words.
///
/// Performance suffers when this is not a power of two.
pub const MAX_STACK: usize = 256;

/// Maximum number of words that a program can output.
///
/// If more values than these are output the oldest are discarded.
/// This behaviour is the same as the stack machines main stack.
/// Performance suffers when this is not a power of two.
pub const MAX_OUTPUT: usize = 16;

/// Hard limit on the number of instructions a program may execute.
pub const MAX_INSTRUCTIONS: usize = u16::MAX as usize;

/// The type used to fill the stack and do calculations with.
///
/// Usually this is a floating point type; some opcodes lose their
/// functionality if `MachineWord` is an integer. Must be signed.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
///
/// Selected at compile time: `f32` by default, `i32` with the `word-i32` feature.
#[cfg(not(feature = "word-i32"))]
pub type MachineWord = f32;

/// The type used to fill the stack and do calculations with.
///
/// Integer variant selected with the `word-i32` feature. See the float
/// variant for canonical documentation.
#[cfg(feature = "word-i32")]
pub type MachineWord = i32;

/// Bytes in a [`MachineWord`] type.
pub const WORD_SIZE: usize = std::mem::size_of::<MachineWord>();

/// Casts a literal or expression to [`MachineWord`] using an `as` cast.
///
/// `mw!(2)` becomes `2 as MachineWord`.
#[macro_export]
macro_rules! mw {
    ($val:expr) => {
        $val as $crate::consts::MachineWord
    };
}

/// Type used to encode bytecode instructions.
///
/// The length in bits of [`Instruction`] must be smaller or equal to that of [`MachineWord`].
/// This is a consequence of the way handling of immediate values works.
/// Parts of the code are not fully tested when this is not [`u8`].
/// A change here also requires changing the `#[repr(type)]` statement for the
/// [`crate::instr::OpCode`] enum.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
pub type Instruction = u8;

/// Bytes in an [`Instruction`] type.
pub const INSTRUCTION_SIZE: usize = std::mem::size_of::<Instruction>();

/// Number of elements of the input program needed to construct one [`MachineWord`] immediate value.
pub const INSTR_PER_WORD: usize = WORD_SIZE / INSTRUCTION_SIZE;

/// Type used to encode jump targets.
///
/// The length in bits of [`Instruction`] must be smaller or equal to that of [`Address`].
pub type Address = u32;

/// Bytes in a [`Address`] type.
pub const ADDRESS_SIZE: usize = std::mem::size_of::<Address>();

/// Number of elements of the input program needed to construct one [`Address`] jump target value.
pub const INSTR_PER_ADDRESS: usize = ADDRESS_SIZE / INSTRUCTION_SIZE;
