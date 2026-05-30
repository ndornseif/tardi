//! General interpreter functionality

use strum::EnumCount as _;

use crate::consts::{INSTR_PER_WORD, Instruction, MAX_OUTPUT, MAX_RUNTIME, MAX_STACK, MachineWord};
use crate::instr::OpCode;
use crate::mw;
use crate::stack::Stack;
#[allow(unused_imports)] // Only needed when [`MachineWord`] is an integer.
use crate::consts::Sqrt as _;
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
    Sub      => op_sub,
    Mul      => op_mul,
    Div      => op_div,
    Max      => op_max,
    Min      => op_min,
    ModDiv   => op_mod_div,
    Sqrt     => op_sqrt,
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
    execution_count: usize,
}

macro_rules! op_one_operand {
    ($name:ident, $method:ident) => {
        fn $name(&mut self) {
            let a = self.stack.pop();
            self.stack.push(a.$method());
        }
    };
    ($name:ident, $op:tt) => {
        fn $name(&mut self) {
            let a = self.stack.pop();
            self.stack.push($op a);
        }
    };
}

macro_rules! op_two_operand {
    ($name:ident, $method:ident) => {
        fn $name(&mut self) {
            let b = self.stack.pop();
            let a = self.stack.pop();
            self.stack.push(a.$method(b));
        }
    };
    ($name:ident, $op:tt) => {
        fn $name(&mut self) {
            let b = self.stack.pop();
            let a = self.stack.pop();
            self.stack.push(a $op b);
        }
    };
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

    /// Return contents of the output stack as Vec.
    pub fn output(&self) -> Vec<MachineWord> {
        self.output.to_vec()
    }

    /// Execute program.
    /// Will end when:
    ///     - Halt instruction called
    ///     - End of program reached
    ///     - [`MAX_RUNTIME`] instructions executed
    pub fn execute(&mut self) {
        while self.execution_count < MAX_RUNTIME {
            self.dispatch();
            if self.halted() {
                break;
            }
        }
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
        self.execution_count += 1;
        DISPATCH_TABLE[(op as usize) % OpCode::COUNT](self);
    }

    #[allow(clippy::unused_self)]
    fn op_nop(&mut self) {}

    fn op_halt(&mut self) {
        self.halted = true;
    }

    fn op_push_imm(&mut self) {
        // TODO: instead of wrapping treat as zero.
        // Add test for this.
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

    op_two_operand!(op_add, +);
    op_two_operand!(op_sub, -);
    op_two_operand!(op_mul, *);
    op_two_operand!(op_div, /);
    op_two_operand!(op_mod_div, %);
    op_two_operand!(op_max, max);
    op_two_operand!(op_min, min);
    op_one_operand!(op_sqrt, sqrt);
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
            MachineWord::from_le_bytes([1, 2, 3, 4]),
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

    macro_rules! test_two_operand_opcode {
        ($name:ident, $val_a:expr, $val_b:expr, $rslt:expr, $opcode:expr, $msg:expr) => {
            #[test]
            fn $name() {
                let program: Vec<u8> = vec![
                    OpCode::PushIn.into(),
                    OpCode::PushIn.into(),
                    $opcode.into(),
                    OpCode::PopOut.into(),
                ];
                let data: Vec<MachineWord> = vec![mw!($val_a), mw!($val_b)];
                let mut int = Interpreter::new_from_program(program, data.clone());
                int.execute();
                assert_eq!(vec![mw!($rslt)], int.output(), $msg);
            }
        };
    }
    test_two_operand_opcode!(
        add_opcode,
        5,
        6,
        11,
        OpCode::Add,
        "addition did not return expected sum"
    );
    test_two_operand_opcode!(
        sub_opcode,
        5,
        6,
        1,
        OpCode::Sub,
        "subtraction did not return expected difference"
    );
    test_two_operand_opcode!(
        mul_opcode,
        5,
        6,
        30,
        OpCode::Mul,
        "multiplication did not return expected product"
    );
    test_two_operand_opcode!(
        div_opcode,
        5,
        30,
        6,
        OpCode::Div,
        "division did not return expected quotiend"
    );
    test_two_operand_opcode!(
        mod_div_opcode,
        7,
        30,
        2,
        OpCode::ModDiv,
        "modulo division did not return expected remainder"
    );
    test_two_operand_opcode!(
        min_opcode,
        5,
        6,
        5,
        OpCode::Min,
        "minimum function did not return expected result"
    );
    test_two_operand_opcode!(
        max_opcode,
        5,
        6,
        6,
        OpCode::Max,
        "maximum function did not return expected result"
    );
    macro_rules! test_one_operand_opcode {
        ($name:ident, $val_a:expr, $rslt:expr, $opcode:expr, $msg:expr) => {
            #[test]
            fn $name() {
                let program: Vec<u8> = vec![
                    OpCode::PushIn.into(),
                    $opcode.into(),
                    OpCode::PopOut.into(),
                ];
                let mut int = Interpreter::new_from_program(program, vec![mw!($val_a)]);
                int.execute();
                assert_eq!(vec![mw!($rslt)], int.output(), $msg);
            }
        };
    }
    test_one_operand_opcode!(sqrt_opcode, 16, 4, OpCode::Sqrt, "square root did not return expected root");
}
