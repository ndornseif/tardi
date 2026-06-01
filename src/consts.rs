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
pub const MAX_INSTRUCTIONS: usize = Address::MAX as usize;

/// The type used to fill the stack and do calculations with.
///
/// Usually this is a floating point type; some opcodes lose their
/// functionality if `MachineWord` is an integer. Must be signed.
/// It has been set to `f32` because neither the `long-word` or the `int-word` features are enabled.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
#[cfg(all(not(feature = "long-word"), not(feature = "int-word")))]
pub type MachineWord = f32;

/// The type used to fill the stack and do calculations with.
///
/// Usually this is a floating point type; some opcodes lose their
/// functionality if `MachineWord` is an integer. Must be signed.
/// It has been set to `i32` because the `int-word` but not the `long-word` features are enabled.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
#[cfg(all(not(feature = "long-word"), feature = "int-word"))]
pub type MachineWord = i32;

/// The type used to fill the stack and do calculations with.
///
/// Usually this is a floating point type; some opcodes lose their
/// functionality if `MachineWord` is an integer. Must be signed.
/// It has been set to `f64` because the `long-word` feature is enabled but `int-word` is not.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
#[cfg(all(feature = "long-word", not(feature = "int-word")))]
pub type MachineWord = f64;

/// The type used to fill the stack and do calculations with.
///
/// Usually this is a floating point type; some opcodes lose their
/// functionality if `MachineWord` is an integer. Must be signed.
/// It has been set to `i64` because both the `long-word` or the `int-word` features are enabled.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
#[cfg(all(feature = "long-word", feature = "int-word"))]
pub type MachineWord = i64;

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
/// A change here also requires changing the `#[repr(type)]` statement for the
/// [`crate::instr::OpCode`] enum.
/// It has been set to `u8` because the `long-instruction` feature is not enabled.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
#[cfg(not(feature = "long-instruction"))]
pub type Instruction = u8;

/// Type used to encode bytecode instructions.
///
/// The length in bits of [`Instruction`] must be smaller or equal to that of [`MachineWord`].
/// This is a consequence of the way handling of immediate values works.
/// A change here also requires changing the `#[repr(type)]` statement for the
/// [`crate::instr::OpCode`] enum.
/// It has been set to `u16` because the `long-instruction` feature is enabled.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
#[cfg(feature = "long-instruction")]
pub type Instruction = u16;

/// Bytes in an [`Instruction`] type.
pub const INSTRUCTION_SIZE: usize = std::mem::size_of::<Instruction>();

/// Number of elements of the input program needed to construct one [`MachineWord`] immediate value.
pub const INSTR_PER_WORD: usize = WORD_SIZE / INSTRUCTION_SIZE;

/// Type used to encode jump targets.
///
/// The length in bits of [`Instruction`] must be smaller or equal to that of [`Address`].
/// This should be set to an unsigned integer.
///
/// It has been set to `u16` because the `long-address` feature is not enabled.
/// This limits the maximum program length to 65535, enable `long-address` for 32 bit addresses.
#[cfg(not(feature = "long-address"))]
pub type Address = u16;

/// Type used to encode jump targets.
///
/// The length in bits of [`Instruction`] must be smaller or equal to that of [`Address`].
/// This should be set to an unsigned integer.
///
/// It has been set to `u32` because the `long-address` feature is enabled.
#[cfg(feature = "long-address")]
pub type Address = u32;

/// Bytes in a [`Address`] type.
pub const ADDRESS_SIZE: usize = std::mem::size_of::<Address>();

/// Number of elements of the input program needed to construct one [`Address`] jump target value.
pub const INSTR_PER_ADDRESS: usize = ADDRESS_SIZE / INSTRUCTION_SIZE;
