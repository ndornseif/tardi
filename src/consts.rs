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
pub const MAX_OUTPUT: usize = 256;

/// Hard limit on the number of instructions a program may execute.
pub const MAX_RUNTIME: usize = u16::MAX as usize;

/// The type used to fill the stack and do calculations with.
///
/// Usually this is a floating point type; some opcodes lose their
/// functionality if `MachineWord` is an integer.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
pub type MachineWord = f32;

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
/// The length in bits of `Instruction` must be smaller or equal to that of `MachineWord`.
/// This is a consequence of the way handling of immediate values works.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
pub type Instruction = u8;

/// Number of elements of the input program needed to construct one `MachineWord` immediate value.
pub const INSTR_PER_WORD: usize =
    std::mem::size_of::<MachineWord>() / std::mem::size_of::<Instruction>();
