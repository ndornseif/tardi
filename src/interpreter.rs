//! General interpreter functionality

use strum::EnumCount as _;

use crate::consts::{INSTR_PER_WORD, Instruction, MAX_OUTPUT, MAX_STACK, MachineWord};
use crate::instr::OpCode;
use crate::mw;
use crate::stack::Stack;
use crate::util::{WrappingGet as _, word_from_instructions};

type OpCodeHandler = fn(&mut Interpreter);

/// Builds the dispatch table where unspecified entries are filled with `op_nop`.
macro_rules! make_dispatch_table {
    ($($variant:ident => $handler:ident),* $(,)?) => {{
        let mut t: [OpCodeHandler; OpCode::COUNT] = [Interpreter::op_nop; OpCode::COUNT];
        $(t[OpCode::$variant as usize] = Interpreter::$handler;)*
        t
    }};
}

/// Maps [`OpCode`] values to the interpreter's operation function handlers.
static DISPATCH_TABLE: [OpCodeHandler; OpCode::COUNT] = make_dispatch_table! {
    Halt     => op_halt,
    PushImm  => op_push_imm,
    PushIn   => op_push_in,
    PushOne  => op_push_one,
    PushZero => op_push_zero,
    PopOut   => op_pop_out,
    Add      => op_add,
};

/// The TARDI stack VM that executes [`OpCode`]s supplied as `Vec<u8>`.
/// Instructions are taken modulo the total number of opcodes,
/// so every byte maps to a valid instruction.
#[derive(Default, Debug, Clone)]
pub struct Interpreter {
    stack: Stack<MachineWord, MAX_STACK>,
    instructions: Vec<Instruction>,
    program_counter: usize,
    input: Vec<MachineWord>,
    output: Stack<MachineWord, MAX_OUTPUT>,
    halted: bool,
}

impl Interpreter {
    /// Initialize a new interpreter with a [`Vec`] of [`Instruction`] bytes,
    /// which encode [`OpCode`]s and their immediate values.
    /// Input data can be supplied as a [`Vec`] of [`MachineWord`]s.
    pub fn new_from_program(program: Vec<Instruction>, input: Vec<MachineWord>) -> Self {
        Self {
            instructions: program,
            input,
            ..Default::default()
        }
    }

    /// Returns `true` if the interpreter has halted.
    pub fn halted(&self) -> bool {
        self.halted
    }

    /// Fetch and execute the instruction pointed to by the `program_counter`.
    ///
    /// If all instructions have been executed the interpreter will halt.
    /// When halted, this function is a no-op.
    pub fn dispatch(&mut self) {
        if self.halted {
            return;
        }
        if self.program_counter >= self.instructions.len() {
            self.halted = true;
            return;
        }
        let op = self.instructions[self.program_counter];
        self.program_counter += 1;
        DISPATCH_TABLE[(op as usize) % OpCode::COUNT](self);
    }

    #[allow(clippy::unused_self)]
    fn op_nop(&mut self) {}

    fn op_halt(&mut self) {
        self.halted = true;
    }

    fn op_push_imm(&mut self) {
        let mut imm_parts = [Instruction::default(); INSTR_PER_WORD];
        for (i, part) in imm_parts.iter_mut().enumerate() {
            *part = self.instructions.wrapping_get(self.program_counter + i);
        }
        let imm = word_from_instructions(imm_parts);
        self.stack.push(imm);
        self.program_counter += INSTR_PER_WORD;
    }

    fn op_push_in(&mut self) {
        if self.input.is_empty() {
            return;
        }
        self.stack.push(self.input.pop().unwrap());
    }

    #[allow(clippy::cast_precision_loss)]
    fn op_push_one(&mut self) {
        self.stack.push(mw!(1));
    }

    fn op_push_zero(&mut self) {
        self.stack.push(MachineWord::default());
    }

    fn op_pop_out(&mut self) {
        self.output.push(self.stack.pop());
    }

    fn op_add(&mut self) {
        let operand_a = self.stack.pop();
        let operand_b = self.stack.pop();
        self.stack.push(operand_a + operand_b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_imm_reads_correct_bytes() {
        let program = vec![OpCode::PushImm as u8, 1, 2, 3, 4];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.dispatch();
        assert_eq!(
            INSTR_PER_WORD + 1,
            int.program_counter,
            "PC should advance past opcode and all immediate bytes"
        );
        assert_eq!(
            f32::from_le_bytes([1, 2, 3, 4]),
            int.stack.peek(),
            "immediate bytes should be decoded onto the stack"
        );
    }

    #[test]
    fn dispatch_sequence() {
        let program: Vec<u8> = vec![
            OpCode::PushIn.into(),
            OpCode::PushOne.into(),
            OpCode::PushZero.into(),
            OpCode::Add.into(),
            OpCode::Add.into(),
        ];
        let data: Vec<MachineWord> = vec![mw!(3)];
        let expected_sum = data[0] + mw!(1);
        let mut int = Interpreter::new_from_program(program, data.clone());
        assert_eq!(
            MachineWord::default(),
            int.stack.peek(),
            "stack should be empty initially"
        );
        int.dispatch(); // PushIn: pushes data[0]
        assert_eq!(
            data[0],
            int.stack.peek(),
            "PushIn should push the input value"
        );
        int.dispatch(); // PushOne
        assert_eq!(mw!(1), int.stack.peek(), "PushOne should push 1");
        int.dispatch(); // PushZero
        assert_eq!(
            MachineWord::default(),
            int.stack.peek(),
            "PushZero should push 0"
        );
        int.dispatch(); // Add: 0 + 1
        assert_eq!(mw!(1), int.stack.peek(), "Add(0, 1) should give 1");
        int.dispatch(); // Add: 1 + data[0]
        assert_eq!(
            expected_sum,
            int.stack.peek(),
            "Add should give the expected sum"
        );
        int.dispatch(); // past end: halts
        assert_eq!(
            expected_sum,
            int.stack.peek(),
            "stack should be unchanged after halting"
        );
        assert!(
            int.halted,
            "interpreter should be halted after running past end of program"
        );
    }
}
