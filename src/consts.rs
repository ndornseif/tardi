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
/// functionality if `MachineWord` is an integer. Must be singned.
///
/// ***All*** possible sequences of bytes (of the correct length) must represent a
/// valid value of this type.
pub type MachineWord = f32;

/// Bytes in an [`MachineWord`] type.
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

/// Formats a [`MachineWord`] immediate value for disassembly output.
/// Uses scientific notation for floats, plain [`Display`](std::fmt::Display) for integers.
pub(crate) trait FormatImm {
    /// Returns a formatted string of the immediate value.
    fn format_imm(&self) -> String;
}

macro_rules! impl_format_imm_float {
    ($($t:ty),+) => {
        $(impl FormatImm for $t {
            fn format_imm(&self) -> String { format!("{self:e}") }
        })+
    };
}
impl_format_imm_float!(f32, f64);

macro_rules! impl_format_imm_int {
    ($($t:ty),+) => {
        $(impl FormatImm for $t {
            fn format_imm(&self) -> String { format!("{self}") }
        })+
    };
}
impl_format_imm_int!(i8, i16, i32, i64, i128, isize);

/// Implement some float only functions for the signed integers.
///
/// This allows us to use signed integers instead of floats
/// as the [`MachineWord`] type.  
/// All execept `sqrt` are just the identity function for ints.
macro_rules! impl_float_funcs_for_int {
    ($($t:ty),+) => {
        $(impl Sqrt for $t {
            fn sqrt(self) -> Self { Self::isqrt(self) }
        }
        impl Round for $t {
            fn round(self) -> Self { self }
        }
        impl Trunc for $t {
            fn trunc(self) -> Self { self }
        }
        impl Floor for $t {
            fn floor(self) -> Self { self }
        }
        impl Ceil for $t {
            fn ceil(self) -> Self { self }
        })+
    };
}
impl_float_funcs_for_int!(i8, i16, i32, i64, i128, isize);

/// Effectivley renaming isqrt to sqrt for the ints.
/// This is done to allow [`MachineWord`] to be int or float.
#[allow(dead_code)]
pub(crate) trait Sqrt {
    #[allow(clippy::return_self_not_must_use)]
    fn sqrt(self) -> Self;
}

#[allow(dead_code)]
pub(crate) trait Round {
    fn round(self) -> Self;
}

#[allow(dead_code)]
pub(crate) trait Trunc {
    fn trunc(self) -> Self;
}

#[allow(dead_code)]
pub(crate) trait Floor {
    fn floor(self) -> Self;
}

#[allow(dead_code)]
pub(crate) trait Ceil {
    fn ceil(self) -> Self;
}
