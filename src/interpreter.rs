//! General interpreter functionality

use strum::EnumCount as _;

use crate::consts::{INSTR_PER_WORD, Instruction, MAX_STACK, MachineWord};
use crate::instr::OpCode;
use crate::util::{WrappingGet as _, word_from_instructions};

#[derive(Debug, Clone)]
pub struct Stack {
    stack: [MachineWord; MAX_STACK],
    total_pushes: usize,
    /// How many elements can be popped before the stack is empty
    /// Since we allow overfilling this can be less than `total_pushes`.
    depth: usize,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            stack: [MachineWord::default(); MAX_STACK],
            total_pushes: 0,
            depth: 0,
        }
    }
}

impl Stack {
    fn len(&self) -> usize {
        self.depth
    }

    fn push(&mut self, value: MachineWord) {
        self.stack[self.total_pushes % MAX_STACK] = value;
        self.depth = MAX_STACK.min(self.depth + 1);
        self.total_pushes += 1;
    }

    fn pop(&mut self) -> MachineWord {
        if self.depth == 0 {
            return MachineWord::default();
        }
        self.total_pushes = self.total_pushes.saturating_sub(1);
        self.depth = self.depth.saturating_sub(1);
        self.stack[self.total_pushes % MAX_STACK]
    }

    fn peek(&self) -> MachineWord {
        if self.depth == 0 {
            return MachineWord::default();
        }
        self.stack[(self.total_pushes - 1) % MAX_STACK]
    }

    fn peek_at(&self, depth: usize) -> MachineWord {
        if depth >= self.len() {
            return MachineWord::default();
        }
        self.stack[(self.total_pushes - 1 - depth) % MAX_STACK]
    }
}

type OpCodeHandler = fn(&mut Interpreter);

/// Builds the dispatch table where unspecified entries are filled with with `op_nop`.
macro_rules! make_dispatch_table {
    ($($variant:ident => $handler:ident),* $(,)?) => {{
        let mut t: [OpCodeHandler; OpCode::COUNT] = [Interpreter::op_nop; OpCode::COUNT];
        $(t[OpCode::$variant as usize] = Interpreter::$handler;)*
        t
    }};
}

pub static DISPATCH_TABLE: [OpCodeHandler; OpCode::COUNT] = make_dispatch_table! {
    Halt     => op_halt,
    PushImm  => op_push_imm,
    PushIn   => op_push_in,
    PushOne  => op_push_one,
    PushZero => op_push_zero,
    Add      => op_add,
};
#[derive(Default, Debug, Clone)]
pub struct Interpreter {
    stack: Stack,
    instructions: Vec<Instruction>,
    program_counter: usize,
    input: Vec<MachineWord>,
    halted: bool,
}

impl Interpreter {
    fn new_from_program(program: Vec<Instruction>, input: Vec<MachineWord>) -> Self {
        Self {
            instructions: program,
            input,
            ..Default::default()
        }
    }

    fn dispatch(&mut self) {
        if self.program_counter >= self.instructions.len() {
            self.halted = true;
            return;
        }
        if self.halted {
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
    fn op_push_one(&mut self) {
        self.stack.push(1 as MachineWord);
    }
    fn op_push_zero(&mut self) {
        self.stack.push(MachineWord::default());
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

    mod stack {
        use super::*;

        #[test]
        fn overflow_wraps() {
            let mut stack = Stack::default();
            // Push five more elements that the stack can accept.
            for i in 0..(MAX_STACK + 5) {
                stack.push(i as MachineWord);
            }
            for i in (5..(MAX_STACK + 5)).rev() {
                assert_eq!(i as MachineWord, stack.pop());
            }
        }

        #[test]
        fn underflow_returns_default() {
            let mut stack = Stack::default();
            stack.push(1 as MachineWord);
            assert_eq!(1 as MachineWord, stack.pop());
            for _ in 0..(2 * MAX_STACK) {
                assert_eq!(MachineWord::default(), stack.pop());
            }
        }

        #[test]
        fn push_pop_peek() {
            let mut stack = Stack::default();
            stack.push(1 as MachineWord);
            stack.push(2 as MachineWord);
            assert_eq!(2 as MachineWord, stack.peek());
            assert_eq!(2 as MachineWord, stack.pop());
            assert_eq!(1 as MachineWord, stack.pop());
            assert_eq!(MachineWord::default(), stack.pop());
        }
    }

    mod interpreter {
        use super::*;

        #[test]
        fn push_imm_reads_correct_bytes() {
            let program = vec![OpCode::PushImm as u8, 1, 2, 3, 4];
            let mut int = Interpreter::new_from_program(program, vec![]);
            int.dispatch();
            assert_eq!(INSTR_PER_WORD + 1, int.program_counter);
            assert_eq!(f32::from_le_bytes([1, 2, 3, 4]), int.stack.peek());
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
            let data: Vec<MachineWord> = vec![3 as MachineWord];
            let expected_sum = data[0] + 1 as MachineWord;
            let mut int = Interpreter::new_from_program(program, data.clone());
            assert_eq!(MachineWord::default(), int.stack.peek());
            int.dispatch(); // PushIn: pushes data[0] (last element)
            assert_eq!(data[0], int.stack.peek());
            int.dispatch(); // PushOne
            assert_eq!(1 as MachineWord, int.stack.peek());
            int.dispatch(); // PushZero
            assert_eq!(MachineWord::default(), int.stack.peek());
            int.dispatch(); // Add: 0 + 1
            assert_eq!(1 as MachineWord, int.stack.peek());
            int.dispatch(); // Add: 1 + data[0]
            assert_eq!(expected_sum, int.stack.peek());
            int.dispatch(); // past end: halts
            assert_eq!(expected_sum, int.stack.peek());
            assert!(int.halted);
        }
    }
}
