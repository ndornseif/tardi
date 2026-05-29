//! General interpreter functionality

use strum::EnumCount;

use crate::consts::{MachineWord, MAX_STACK, Instruction, INSTR_PER_WORD};
use crate::instr::OpCode;
use crate::util::{WrappingGet, word_from_instructions};

pub struct Stack {
    stack: [MachineWord; MAX_STACK],
    stack_pointer: usize,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            stack: [MachineWord::default(); MAX_STACK],
            stack_pointer: 0,
        }
    }
}

impl Stack {
    fn pointer_add(&mut self, value: usize) {
        self.stack_pointer = (self.stack_pointer + value) % MAX_STACK;
    }

    fn pointer_sub(&mut self, value: usize) {
        self.stack_pointer = (self.stack_pointer as isize - value as isize).rem_euclid(MAX_STACK as isize) as usize;
    }

    fn push(&mut self, value: MachineWord) {
        self.pointer_add(1);
        self.stack[self.stack_pointer] = value;
    }

    fn peek(&self) -> MachineWord {
        self.stack[self.stack_pointer]
    }
}

type OpCodeHandler = fn(&mut Interpreter);

static DISPATCH: [OpCodeHandler; OpCode::COUNT] = {
    let mut t: [OpCodeHandler; OpCode::COUNT] = [Interpreter::op_nop; OpCode::COUNT];
    t[OpCode::Halt as usize] = Interpreter::op_halt;
    t[OpCode::PushImm as usize] = Interpreter::op_push_imm;
    t
};

#[derive(Default)]
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
    fn clock(&mut self) {
        if self.halted {
            return;
        }
    }

    fn dispactch(&mut self) {
        
    }

    fn op_nop(&mut self) {}
    fn op_halt(&mut self) { self.halted = true; }
    fn op_push_imm(&mut self) {
        let mut imm_parts = [Instruction::default(); INSTR_PER_WORD];
        for i in 0..INSTR_PER_WORD {
            imm_parts[i] = self.instructions.wrapping_get(self.program_counter + 1 + i);
        }
        let imm = word_from_instructions(imm_parts);
        self.stack.push(imm);
        self.program_counter += INSTR_PER_WORD;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stack_pointer_add() {
        let mut stack = Stack::default();
        assert_eq!(0, stack.stack_pointer);
        stack.pointer_add(2);
        assert_eq!(2, stack.stack_pointer);
        stack.pointer_add(MAX_STACK - 1);
        assert_eq!(1, stack.stack_pointer);
    }

    #[test]
    fn stack_pointer_sub() {
        let mut stack = Stack::default();
        assert_eq!(0, stack.stack_pointer);
        stack.pointer_add(2);
        stack.pointer_sub(1);
        assert_eq!(1, stack.stack_pointer);
        stack.pointer_sub(MAX_STACK - 1);
        assert_eq!(2, stack.stack_pointer);
    }

    #[test]
    fn push_imm() {
        let program = vec![0_u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.op_push_imm();
        assert_eq!(INSTR_PER_WORD, int.program_counter);
        assert_eq!(1.5399896e-36, int.stack.peek());
    }
}

