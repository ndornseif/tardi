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
#[allow(dead_code)]
pub trait Sqrt {
    #[allow(clippy::return_self_not_must_use)]
    fn sqrt(self) -> Self;
}

/// Rounding to nearest integer. Identity function for integer types.
#[allow(dead_code)]
pub trait Round {
    fn round(self) -> Self;
}

/// Truncation toward zero. Identity function for integer types.
#[allow(dead_code)]
pub trait Trunc {
    fn trunc(self) -> Self;
}

/// Floor (toward negative infinity). Identity function for integer types.
#[allow(dead_code)]
pub trait Floor {
    fn floor(self) -> Self;
}

/// Ceiling (toward positive infinity). Identity function for integer types.
#[allow(dead_code)]
pub trait Ceil {
    fn ceil(self) -> Self;
}

/// Machine epsilon for the type. Zero for integer types.
#[allow(dead_code)]
pub trait Epsilon {
    /// The machine epsilon value.
    const EPSILON: Self;
}

/// Whether a value is finite. Always `true` for integer types.
#[allow(dead_code)]
pub trait IsFinite {
    #[allow(clippy::wrong_self_convention)]
    fn is_finite(self) -> bool;
}

/// Implement float-only functions for signed integers so they can be used as
/// [`MachineWord`](crate::consts::MachineWord).
macro_rules! impl_float_funcs_for_int {
    ($($t:ty),+) => {
        $(
        impl Epsilon for $t {
            const EPSILON: Self = 0;
        }
        impl Sqrt for $t {
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
        }
        impl IsFinite for $t {
            fn is_finite(self) -> bool { true }
        }
        )+
    };
}
impl_float_funcs_for_int!(i8, i16, i32, i64, i128, isize);

/// Type appropriate test inputs and expected outputs for opcode test
/// that have behaviour that depends on the type of [`MachineWord`].
#[cfg(test)]
pub trait TestLiterals: Sized {
    const ROUND_INPUT: Self;
    const ROUND_EXPECTED: Self;
    const TRUNC_INPUT: Self;
    const TRUNC_EXPECTED: Self;
    const CEIL_INPUT: Self;
    const CEIL_EXPECTED: Self;
    const FLOOR_INPUT: Self;
    const FLOOR_EXPECTED: Self;
}

macro_rules! impl_test_literals_float {
    ($($t:ty),+) => {$(
        #[cfg(test)]
        impl TestLiterals for $t {
            const ROUND_INPUT: Self = 5.49;
            const ROUND_EXPECTED: Self = 5.0;
            const TRUNC_INPUT: Self = 5.99;
            const TRUNC_EXPECTED: Self = 5.0;
            const CEIL_INPUT: Self = 5.01;
            const CEIL_EXPECTED: Self = 6.0;
            const FLOOR_INPUT: Self = -5.01;
            const FLOOR_EXPECTED: Self = -6.0;
        }
    )+};
}
impl_test_literals_float!(f32, f64);

macro_rules! impl_test_literals_int {
    ($($t:ty),+) => {$(
        #[cfg(test)]
        impl TestLiterals for $t {
            const ROUND_INPUT: Self = 5;
            const ROUND_EXPECTED: Self = 5;
            const TRUNC_INPUT: Self = 5;
            const TRUNC_EXPECTED: Self = 5;
            const CEIL_INPUT: Self = 5;
            const CEIL_EXPECTED: Self = 5;
            const FLOOR_INPUT: Self = -5;
            const FLOOR_EXPECTED: Self = -5;
        }
    )+};
}
impl_test_literals_int!(i8, i16, i32, i64, i128, isize);
