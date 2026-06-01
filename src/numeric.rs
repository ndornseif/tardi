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
