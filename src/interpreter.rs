//! General interpreter functionality

use strum::EnumCount as _;

#[allow(unused_imports)] // Only needed when [`MachineWord`] is an integer.
use crate::consts::{Ceil as _, Floor as _, Round as _, Sqrt as _, Trunc as _};

use crate::consts::{
    INSTR_PER_ADDRESS, INSTR_PER_WORD, Instruction, MAX_OUTPUT, MAX_RUNTIME, MAX_STACK, MachineWord,
};
use crate::instr::OpCode;
use crate::mw;
use crate::stack::Stack;
use crate::util::{WrappingGet as _, address_from_instructions, word_from_instructions};

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
    Round    => op_round,
    Trunc    => op_trunc,
    Ceil     => op_ceil,
    Floor    => op_floor,
    Neg      => op_neg,
    Abs      => op_abs,
    Swap     => op_swap,
    Remove   => op_remove,
    Dup      => op_dup,
    Jmp      => op_jmp,
    JmpZero  => op_jmp_zero,
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
        // Adapt like code used in jmp instructions.
        // Update relevant docs.
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

    fn op_swap(&mut self) {
        let a = self.stack.pop();
        let b = self.stack.pop();
        self.stack.push(a);
        self.stack.push(b);
    }

    fn op_remove(&mut self) {
        let _ = self.stack.pop();
    }

    fn op_dup(&mut self) {
        self.stack.push(self.stack.peek());
    }

    fn op_jmp(&mut self) {
        let mut addr_parts = [Instruction::default(); INSTR_PER_WORD];
        for (i, part) in addr_parts.iter_mut().enumerate() {
            *part = self
                .instructions
                .get(self.program_counter + i)
                .copied()
                .unwrap_or_default();
        }
        let addr = address_from_instructions(addr_parts) as usize;
        self.program_counter = addr % self.instructions.len();
    }

    fn op_jmp_zero(&mut self) {
        // TODO: Reduce code repetition in jump instructions.
        let mut addr_parts = [Instruction::default(); INSTR_PER_WORD];
        for (i, part) in addr_parts.iter_mut().enumerate() {
            *part = self
                .instructions
                .get(self.program_counter + i)
                .copied()
                .unwrap_or_default();
        }
        let addr = address_from_instructions(addr_parts) as usize;
        #[allow(clippy::float_cmp)]
        if self.stack.peek() == MachineWord::default() {
            self.program_counter = addr % self.instructions.len();
        } else {
            self.program_counter += INSTR_PER_ADDRESS;
        }
    }

    op_two_operand!(op_add, +);
    op_two_operand!(op_sub, -);
    op_two_operand!(op_mul, *);
    op_two_operand!(op_div, /);
    op_two_operand!(op_mod_div, %);
    op_two_operand!(op_max, max);
    op_two_operand!(op_min, min);

    op_one_operand!(op_sqrt, sqrt);
    op_one_operand!(op_round, round);
    op_one_operand!(op_trunc, trunc);
    op_one_operand!(op_ceil, ceil);
    op_one_operand!(op_floor, floor);
    op_one_operand!(op_neg, -);
    op_one_operand!(op_abs, abs);
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::consts::Address;
    use crate::util::instructions_from_address;

    #[test]
    fn push_imm_reads_correct_bytes() {
        // TODO: Adapt this for cases where `MachineWord` is not four bytes long.
        let program = vec![OpCode::PushImm as Instruction, 1, 2, 3, 4];
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
    fn basic_jmp() {
        let mut program: Vec<Instruction> = vec![
            OpCode::PushOne.into(),
            OpCode::PushOne.into(),
            OpCode::Jmp.into(),
        ];
        // Jump address is set to skip `Add`.
        // If we dont jump far enough `Add` will be executed and the output becomes two.
        // If we jump to far `PopOut` wont be executed and the output remains empty.
        program.extend_from_slice(&instructions_from_address(
            (program.len() + INSTR_PER_ADDRESS + 1) as Address,
        ));
        program.extend_from_slice(&[
            OpCode::Add.into(),
            OpCode::PopOut.into(),
            OpCode::Halt.into(),
        ]);
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            vec![mw!(1)],
            int.output(),
            "jump did not correctly skip instruction"
        );
    }

    #[test]
    fn jmp_zero() {
        let mut program: Vec<Instruction> = vec![
            OpCode::PushOne.into(),
            OpCode::PushOne.into(),
            OpCode::JmpZero.into(),
        ];
        // Jump address is set to skip `Add`.
        // If we dont jump far enough `Add` will be executed and the output becomes two.
        // If we jump to far `PopOut` wont be executed and the output remains empty.
        // Since TOS is one JmpZero should not be taken.
        //  -> Add is executed, output becomes two.
        program.extend_from_slice(&instructions_from_address(
            (program.len() + INSTR_PER_ADDRESS + 1) as Address,
        ));
        program.extend_from_slice(&[
            OpCode::Add.into(),
            OpCode::PopOut.into(),
            OpCode::Halt.into(),
        ]);
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            vec![mw!(2)],
            int.output(),
            "jump if zero was not correcly ignored"
        );

        let mut program: Vec<Instruction> = vec![
            OpCode::PushOne.into(),
            OpCode::PushOne.into(),
            OpCode::Add.into(),
            OpCode::PushZero.into(),
            OpCode::JmpZero.into(),
        ];
        // Jump address is set to skip `Add`.
        // If we dont jump far enough `Add` will be executed and the output becomes two.
        // If we jump to far `PopOut` wont be executed and the output remains empty.
        // Since TOS is zero JmpZero should be taken.
        //  -> Add is not executed, output becomes zero.
        program.extend_from_slice(&instructions_from_address(
            (program.len() + INSTR_PER_ADDRESS + 1) as Address,
        ));
        program.extend_from_slice(&[
            OpCode::Add.into(),
            OpCode::PopOut.into(),
            OpCode::Halt.into(),
        ]);
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            vec![MachineWord::default()],
            int.output(),
            "jump if zero was not taken"
        );
    }
    #[test]
    fn dispatch_sequence() {
        let program: Vec<Instruction> = vec![
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
                let program: Vec<Instruction> = vec![
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
                let program: Vec<Instruction> =
                    vec![OpCode::PushIn.into(), $opcode.into(), OpCode::PopOut.into()];
                let mut int = Interpreter::new_from_program(program, vec![mw!($val_a)]);
                int.execute();
                assert_eq!(vec![mw!($rslt)], int.output(), $msg);
            }
        };
    }
    test_one_operand_opcode!(
        sqrt_opcode,
        16,
        4,
        OpCode::Sqrt,
        "square root did not return expected root"
    );
    test_one_operand_opcode!(
        round_opcode,
        5.49,
        5,
        OpCode::Round,
        "rounding function did not return expected value"
    );
    test_one_operand_opcode!(
        trunc_opcode,
        5.99,
        5,
        OpCode::Trunc,
        "truncation function did not return expected value"
    );
    test_one_operand_opcode!(
        ceil_opcode,
        5.01,
        6,
        OpCode::Ceil,
        "ceiling function did not return expected value"
    );
    test_one_operand_opcode!(
        floor_opcode,
        -5.01,
        -6,
        OpCode::Floor,
        "floor function did not return expected value"
    );
    test_one_operand_opcode!(
        neg_opcode,
        6,
        -6,
        OpCode::Neg,
        "negation function did not return expected value"
    );
    test_one_operand_opcode!(
        abs_opcode,
        -4,
        4,
        OpCode::Abs,
        "abs function did not return expected value"
    );
}
