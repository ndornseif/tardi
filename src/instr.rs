//! # Instruction definitions
//!
//!
//! ## Jumps
//! All jumps are executed modulo the program size, it is impossible to jump out of the program.
//! A jump destination address is always express as an unsigned integer type defined by [`Address`].

use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{Display, EnumCount};

#[allow(unused_imports)]
use crate::consts::{Address, MachineWord};

/// Represents a possible bytecode instruction for the interpreter.
#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, TryFromPrimitive, IntoPrimitive, EnumCount, Display,
)]
pub enum OpCode {
    /// Does nothing.
    #[default]
    Nop,
    /// Stop execution.
    Halt,
    /// Push immediate to stack.
    PushImm,
    /// Pop value from input and push it onto stack.
    /// Nop if input is empty.
    PushIn,
    /// Push the literal value 1 to the stack.
    PushOne,
    /// Push the [`MachineWord`] default value to stack.  
    /// This is zero for the numeric types.
    PushZero,
    /// Pop the top value from the stack and push it onto the output stack.
    PopOut,
    /// Pop the top two values from the stack and push their sum.
    Add,
    /// Pop the top two values from the stack and push their difference.
    /// The top value is subtracted from the second one.
    Sub,
    /// Pop the top two values from the stack and push their product.
    Mul,
    /// Pop the top two values from the stack and push their quotient.
    /// The second value is divided by the top one.
    Div,
    /// Pop the top two values from the stack and push their minimum.
    Min,
    /// Pop the top two values from the stack and push their maximum.
    Max,
    /// Pop the top two values from the stack and push their remainder.
    /// The second value is modulo divided by the top one.
    ModDiv,
    /// Pop the top value from the stack and push its square root.
    Sqrt,
    /// Pop the top value from the stack and push its rounded value.
    /// Nop if [`MachineWord`] is an integer.
    Round,
    /// Pop the top value from the stack and push its truncated value.
    /// Nop if [`MachineWord`] is an integer.
    Trunc,
    /// Pop the top value from the stack and push its ceiling value.
    /// Nop if [`MachineWord`] is an integer.
    Ceil,
    /// Pop the top value from the stack and push its floor value.
    /// Nop if [`MachineWord`] is an integer.
    Floor,
    /// Pop the top value from the stack and push its negated value.
    Neg,
    /// Pop the top value from the stack and push its absolute value.
    Abs,
    /// Pop the top two values from the stack and push them back in reverse order.
    Swap,
    /// Pop the top value from the stack and drop it.
    Remove,
    /// Push another copy of the top value onto the stack.
    Dup,
    /// Jump to immediate address. Addresses must be unsigned integers.
    Jmp,
    /// Jump to immediate address if top of stack is zero.
    /// Does not pop the value.
    JmpZero,
    /// Jump to immediate address if top of stack is within ten times [`MachineWord::EPSILON`] of zero.
    /// Identical to `JmpZero` if [`MachineWord`] is an integer type.
    /// Does not pop the value.
    JmpAprxZero,
    /// Jump to immediate address if top of stack is positive.
    /// Does not pop the value.
    JmpPos,
    /// Jump to immediate address if top of stack is finite.
    /// Does not pop the value.
    /// Will always jump if [`MachineWord`] is an integer.
    JmpFin,
    /// Jump to address as specified by top of stack.
    /// The value is converted to [`usize`] using Rust's `as` cast.
    /// If the conversion to [`usize`] is impossible because the float is not finite
    /// the jump will be skipped.
    /// This instruction does pop the value from the top of the stack and drops it,
    /// even if the jump was skipped.
    JmpTos,
}
