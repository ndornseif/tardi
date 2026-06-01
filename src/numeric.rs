//! Numeric traits for [`MachineWord`](crate::consts::MachineWord) types.

/// Formats an immediate value for disassembly output.
/// Uses scientific notation for floats, plain [`Display`](std::fmt::Display) for integers.
pub trait FormatImm {
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

/// Effectively renaming [`isqrt`](i32::isqrt) to `sqrt` for integer types,
/// allowing [`MachineWord`](crate::consts::MachineWord) to be int or float.
/// Not needed for float types since they have an inherent `sqrt` method.
#[cfg(feature = "int-word")]
pub trait Sqrt {
    #[allow(clippy::return_self_not_must_use)]
    fn sqrt(self) -> Self;
}

#[cfg(feature = "int-word")]
macro_rules! impl_sqrt_for_int {
    ($($t:ty),+) => {
        $(impl Sqrt for $t {
            fn sqrt(self) -> Self { Self::isqrt(self) }
        })+
    };
}

#[cfg(feature = "int-word")]
impl_sqrt_for_int!(i8, i16, i32, i64, i128, isize);
