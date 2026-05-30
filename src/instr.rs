//! # Instruction definitions
//!
//!
//! ## Jumps
//! All jumps are executed modulo the program size, it is impossible to jump out of the program.

use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::EnumCount;

#[allow(unused_imports)]
use crate::consts::MachineWord;

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, TryFromPrimitive, IntoPrimitive, EnumCount,
)]
pub enum OpCode {
    /// Does nothing.
    #[default]
    Nop,
    /// Stop execution.
    Halt,
    /// Push immediate to stack.
    PushImm,
    /// Pop value from input and push onto stack.
    /// Nop if input is empty.
    PushIn,
    /// Push the value one to the stack.
    PushOne,
    /// Push the [`MachineWord`] default value to stack.  
    /// This is zero for the numeric types.
    PushZero,
    /// Pop the top two values from the stack and push their sum.
    Add,
    /// Pop the top two values from the stack and push their difference.
    /// The second value is subtracted from the first one.
    Sub,
    /// Pop the top two values from the stack and push their product.
    Mul,
    /// Pop the top two values from the stack and push their quotient.
    /// The first value is divided by the second one.
    Div,
    /// Pop the top two values from the stack and push their minimum.
    Min,
    /// Pop the top two values from the stack and push their maximum.
    Max,
    /// Pop the top two values from the stack and push their remainder.
    /// The first value is modulo divided by the second one.
    ModDiv,
    /// Pop top value from the stack and push its square root.
    Sqrt,
    /// Pop top value from the stack and push its rounded value.
    Round,
    /// Pop top value from the stack and push its truncated value.
    Trunc,
    /// Pop top value from the stack and push its negated value.
    Neg,
    /// Pop top value from the stack and push its absolute value.
    Abs,
    /// Pop top two values from the stack and push them back in reverse order.
    Swap,
    /// Pop the top value from the stack and drop it.
    Remove,
    /// Push another copy of the top value onto the stack.
    Dup,
    /// Jump to immediate address.
    Jmp,
    /// Jump to immediate address if top of stack is zero.
    /// Does not pop the value.
    JmpZero,
    /// Jump to immediate address if top of stack is smaller than ten epsilon.
    /// Identical to `JmpZero` if [`MachineWord`] is integer.
    /// Does not pop the value.
    JmpAprxZero,
    /// Jump to immediate address if top of stack is positive.
    /// Does not pop the value.
    JmpPos,
    /// Jump to address as specified by top of stack.
    /// The value is converted to [`usize`] using rusts `as` casts.
    /// This does pop the value from the top of the stack and drop it.
    JmpTos,
}

