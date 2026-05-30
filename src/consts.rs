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

/// Numeric square root that dispatches to [`isqrt`](u32::isqrt) for integers.
///
/// Effectivley renaming isqrt to sqrt for the ints.
/// This is done to allow [`MachineWord`] to be int or float.
pub trait Sqrt {
    /// Returns the square root of `self`.
    #[allow(clippy::return_self_not_must_use)]
    fn sqrt(self) -> Self;
}

// Currently unused, floats already implement a sqrt trait called Sqrt.
#[allow(unused_macros)]
macro_rules! impl_sqrt_float {
    ($($t:ty),+) => {
        $(impl Sqrt for $t {
            #[inline]
            fn sqrt(self) -> Self { Self::sqrt(self) }
        })+
    };
}

macro_rules! impl_sqrt_int {
    ($($t:ty),+) => {
        $(impl Sqrt for $t {
            #[inline]
            fn sqrt(self) -> Self { Self::isqrt(self) }
        })+
    };
}
impl_sqrt_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

/// Formats a [`MachineWord`] immediate value for disassembly output.
/// Uses scientific notation for floats, plain [`Display`](std::fmt::Display) for integers.
pub trait FormatImm {
    /// Returns a formatted string of the immediate value.
    fn format_imm(&self) -> String;
}

impl FormatImm for f32 {
    fn format_imm(&self) -> String { format!("{self:e}") }
}

impl FormatImm for f64 {
    fn format_imm(&self) -> String { format!("{self:e}") }
}

macro_rules! impl_format_imm_int {
    ($($t:ty),+) => {
        $(impl FormatImm for $t {
            fn format_imm(&self) -> String { format!("{self}") }
        })+
    };
}
impl_format_imm_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
