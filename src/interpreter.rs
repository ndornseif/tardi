//! General interpreter functionality

use strum::EnumCount as _;

#[allow(unused_imports)] // Only needed when [`MachineWord`] is an integer.
use crate::numeric::{Ceil as _, Epsilon as _, Floor as _, IsFinite as _, Round as _, Sqrt as _, Trunc as _};

use crate::consts::{
    INSTR_PER_ADDRESS, INSTR_PER_WORD, Instruction, MAX_INSTRUCTIONS, MAX_OUTPUT, MAX_STACK,
    MachineWord,
};
use crate::instr::OpCode;
use crate::mw;
use crate::stack::Stack;
use crate::util::{address_from_instructions, word_from_instructions};

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
    JmpAprxZero => op_jmp_aprx_zero,
    JmpPos      => op_jmp_pos,
    JmpFin      => op_jmp_fin,
};

/// Specifies the reason why Interpreter was halted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HaltReason {
    /// Interpreter has not been halted or no reason defined.
    #[default]
    Undefined,
    /// `Halt` instruction was executed.
    HaltInstruction,
    /// End of program was reached.
    EndOfProgram,
    /// `MAX_INSTRUCTIONS` instructions have been executed.
    MaxInstructions,
}

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
    halt_reason: HaltReason,
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

    /// Returns reason why interpreter was halted.
    /// Will return `None` if interpreter is not halted.
    pub fn halt_reason(&self) -> Option<HaltReason> {
        self.halted.then_some(self.halt_reason)
    }

    /// Return contents of the output stack as Vec.
    pub fn output(&self) -> Vec<MachineWord> {
        self.output.to_vec()
    }

    /// Execute program.
    /// Will end when:
    ///     - Halt instruction called
    ///     - End of program reached
    ///     - [`MAX_INSTRUCTIONS`] instructions executed
    pub fn execute(&mut self) {
        while self.execution_count < MAX_INSTRUCTIONS {
            self.dispatch();
            if self.halted() {
                return;
            }
        }
        self.halted = true;
        self.halt_reason = HaltReason::MaxInstructions;
    }

    /// Fetch and execute the instruction at current PC.
    ///
    /// If all instructions have been executed the interpreter will halt.
    /// When halted, this function is a no-op.
    pub fn dispatch(&mut self) {
        if self.halted {
            return;
        }
        if self.program_counter >= self.instructions.len() {
            self.halted = true;
            self.halt_reason = HaltReason::EndOfProgram;
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
        self.halt_reason = HaltReason::HaltInstruction;
    }

    fn op_push_imm(&mut self) {
        let mut imm_parts = [Instruction::default(); INSTR_PER_WORD];
        for (i, part) in imm_parts.iter_mut().enumerate() {
            *part = self
                .instructions
                .get(self.program_counter + i)
                .copied()
                .unwrap_or_default();
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

    /// Read `INSTR_PER_ADDRESS` bytes from the current PC and decode them as a jump target.
    /// Returns the target address taken modulo program length.
    /// Bytes that extend past the end of the program are treated as zero.
    fn read_address_immediate(&self) -> usize {
        let mut addr_parts = [Instruction::default(); INSTR_PER_ADDRESS];
        for (i, part) in addr_parts.iter_mut().enumerate() {
            *part = self
                .instructions
                .get(self.program_counter + i)
                .copied()
                .unwrap_or_default();
        }
        address_from_instructions(addr_parts) as usize % self.instructions.len()
    }

    fn op_jmp(&mut self) {
        self.program_counter = self.read_address_immediate();
    }

    //TODO: Macro for all these jmp implementations
    fn op_jmp_zero(&mut self) {
        let addr = self.read_address_immediate();
        #[allow(clippy::float_cmp)]
        if self.stack.peek() == MachineWord::default() {
            self.program_counter = addr;
        } else {
            self.program_counter += INSTR_PER_ADDRESS;
        }
    }

    fn op_jmp_aprx_zero(&mut self) {
        let addr = self.read_address_immediate();
        if self.stack.peek().abs() <= (MachineWord::EPSILON * mw!(10)) {
            self.program_counter = addr;
        } else {
            self.program_counter += INSTR_PER_ADDRESS;
        }
    }

    fn op_jmp_pos(&mut self) {
        let addr = self.read_address_immediate();
        if self.stack.peek() > MachineWord::default() {
            self.program_counter = addr;
        } else {
            self.program_counter += INSTR_PER_ADDRESS;
        }
    }

    fn op_jmp_fin(&mut self) {
        let addr = self.read_address_immediate();
        if self.stack.peek().is_finite() {
            self.program_counter = addr;
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
    use crate::numeric::TestLiterals as _;
    use crate::util::{instructions_from_address, instructions_from_word};

    /// Build a program with a jump whose target is not yet known, returning the
    /// offset in the program where the address bytes should be patched.
    fn push_jmp(program: &mut Vec<Instruction>, opcode: OpCode) -> usize {
        program.push(opcode.into());
        let offset = program.len();
        program.extend_from_slice(&instructions_from_address(0));
        offset
    }

    /// Patch a previously reserved jump address slot with the given target.
    fn patch_jmp(program: &mut Vec<Instruction>, offset: usize, target: usize) {
        program[offset..offset + INSTR_PER_ADDRESS]
            .copy_from_slice(&instructions_from_address(target as Address));
    }

    #[test]
    fn halt_on_halt_instruction() {
        let program = vec![OpCode::Nop.into(), OpCode::Halt.into()];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            Some(HaltReason::HaltInstruction),
            int.halt_reason(),
            "incorrect halt reason after halt instruction"
        );
    }

    #[test]
    fn halt_on_program_end() {
        let program = vec![OpCode::Nop.into(), OpCode::Nop.into()];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            Some(HaltReason::EndOfProgram),
            int.halt_reason(),
            "incorrect halt reason after reaching end of program"
        );
    }

    #[test]
    fn halt_on_instruction_limit() {
        let mut program = vec![OpCode::Nop.into()];
        // Constructing an infinite loop.
        let offset = push_jmp(&mut program, OpCode::Jmp);
        patch_jmp(&mut program, offset, 0);
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            Some(HaltReason::MaxInstructions),
            int.halt_reason(),
            "incorrect halt reason when reaching instruction limit"
        );
    }

    #[test]
    fn push_imm_reads_correct_bytes() {
        let expected: MachineWord = mw!(1234);
        let mut program = vec![OpCode::PushImm as Instruction];
        program.extend_from_slice(&instructions_from_word(expected));
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.dispatch();
        assert_eq!(
            INSTR_PER_WORD + 1,
            int.program_counter,
            "PC has not advanced past opcode and all immediate bytes"
        );
        assert_eq!(
            expected,
            int.stack.peek(),
            "immediate value not decoded onto the stack"
        );
    }

    #[test]
    fn push_imm_out_of_bounds_bytes_are_zero() {
        // PushImm with no following bytes: missing immediate bytes default to zero.
        let program = vec![OpCode::PushImm as Instruction];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.dispatch();
        assert_eq!(
            MachineWord::default(),
            int.stack.peek(),
            "out-of-bounds immediate bytes not treated as zero"
        );
    }

    #[test]
    fn basic_jmp() {
        // `Jmp` should skip over add instruction leaving ine as top of stack.
        let mut program: Vec<Instruction> = vec![OpCode::PushOne.into(), OpCode::PushOne.into()];
        let jmp_offset = push_jmp(&mut program, OpCode::Jmp);
        program.push(OpCode::Add.into()); // dead code: jumped over
        let pop_pos = program.len();
        program.extend_from_slice(&[OpCode::PopOut.into(), OpCode::Halt.into()]);
        patch_jmp(&mut program, jmp_offset, pop_pos);

        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            vec![mw!(1)],
            int.output(),
            "jmp did not skip add instruction"
        );
    }

    #[test]
    fn jmp_zero_not_taken() {
        // TOS is one, so `JmpZero` should not be taken.
        // Add runs, output = [1+1] = [2].
        let mut program: Vec<Instruction> = vec![OpCode::PushOne.into(), OpCode::PushOne.into()];
        let jmp_offset = push_jmp(&mut program, OpCode::JmpZero);
        program.push(OpCode::Add.into());
        let pop_pos = program.len();
        program.extend_from_slice(&[OpCode::PopOut.into(), OpCode::Halt.into()]);
        patch_jmp(&mut program, jmp_offset, pop_pos);

        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            vec![mw!(2)],
            int.output(),
            "JmpZero did jump when TOS is non-zero"
        );
    }

    #[test]
    fn jmp_zero_taken() {
        // TOS is zero, `JmpZero` jumps past `PushOne`.
        // TOS remains zero.
        let mut program: Vec<Instruction> = vec![OpCode::PushZero.into()];
        let jmp_offset = push_jmp(&mut program, OpCode::JmpZero);
        program.push(OpCode::PushOne.into()); // dead code: jumped over
        let pop_pos = program.len();
        program.extend_from_slice(&[OpCode::PopOut.into(), OpCode::Halt.into()]);
        patch_jmp(&mut program, jmp_offset, pop_pos);

        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(
            vec![mw!(0)],
            int.output(),
            "JmpZero did not jump when TOS is zero"
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
            "`PushIn` did not push the input value"
        );
        int.dispatch(); // PushOne
        assert_eq!(mw!(1), int.stack.peek(), "PushOne did not push one");
        int.dispatch(); // PushZero
        assert_eq!(
            MachineWord::default(),
            int.stack.peek(),
            "`PushZero` did not push 0"
        );
        int.dispatch(); // Add: 0 + 1
        assert_eq!(mw!(1), int.stack.peek(), "add(0, 1) did not give one");
        int.dispatch(); // Add: 1 + 3
        assert_eq!(
            expected_sum,
            int.stack.peek(),
            "addition did not create expected sum"
        );
        int.dispatch(); // End of program
        assert_eq!(
            expected_sum,
            int.stack.peek(),
            "stack not unchanged after halting"
        );
        assert!(
            int.halted,
            "interpreter not halted after past end of program"
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
        MachineWord::ROUND_INPUT,
        MachineWord::ROUND_EXPECTED,
        OpCode::Round,
        "rounding function did not return expected value"
    );
    test_one_operand_opcode!(
        trunc_opcode,
        MachineWord::TRUNC_INPUT,
        MachineWord::TRUNC_EXPECTED,
        OpCode::Trunc,
        "truncation function did not return expected value"
    );
    test_one_operand_opcode!(
        ceil_opcode,
        MachineWord::CEIL_INPUT,
        MachineWord::CEIL_EXPECTED,
        OpCode::Ceil,
        "ceiling function did not return expected value"
    );
    test_one_operand_opcode!(
        floor_opcode,
        MachineWord::FLOOR_INPUT,
        MachineWord::FLOOR_EXPECTED,
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

    #[test]
    fn swap_opcode() {
        let program: Vec<Instruction> = vec![
            OpCode::PushIn.into(),
            OpCode::PushIn.into(),
            OpCode::Swap.into(),
            OpCode::PopOut.into(),
            OpCode::Halt.into(),
        ];
        let mut int = Interpreter::new_from_program(program, vec![mw!(3), mw!(7)]);
        int.execute();
        assert_eq!(vec![mw!(7)], int.output(), "swap did not reorder elements");
    }

    #[test]
    fn remove_opcode() {
        let program: Vec<Instruction> = vec![
            OpCode::PushIn.into(),
            OpCode::PushIn.into(),
            OpCode::Remove.into(),
            OpCode::PopOut.into(),
            OpCode::Halt.into(),
        ];
        let mut int = Interpreter::new_from_program(program, vec![mw!(3), mw!(7)]);
        int.execute();
        assert_eq!(
            vec![mw!(7)],
            int.output(),
            "remove did not drop the top stack element"
        );
    }

    #[test]
    fn dup_opcode() {
        let program: Vec<Instruction> = vec![
            OpCode::PushIn.into(),
            OpCode::Dup.into(),
            OpCode::PopOut.into(),
            OpCode::PopOut.into(),
            OpCode::Halt.into(),
        ];
        let mut int = Interpreter::new_from_program(program, vec![mw!(5)]);
        int.execute();
        assert_eq!(
            vec![mw!(5), mw!(5)],
            int.output(),
            "Dup did not produce two copies of the top stack element"
        );
    }
}
