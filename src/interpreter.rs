//! General interpreter functionality

use strum::EnumCount as _;

use crate::consts::{
    Address, INSTR_PER_ADDRESS, INSTR_PER_WORD, Instruction, MAX_INSTRUCTIONS, MAX_OUTPUT,
    MAX_STACK, MachineWord,
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
#[cfg(not(feature = "int-word"))]
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
    JmpTos      => op_jmp_tos,
};

/// Maps [`OpCode`] values to the interpreter's operation function handlers.
/// This is the int version of the mapping that turns some float specific
/// instructions like `Ceil` or `Round` into `Nop`.  
/// It also maps `JmpAprxZero` to `JmpZero` and `JmpFin` to `Jmp` since all int are finite
/// and zero is the only int thats approximatley equal to zero.
#[cfg(feature = "int-word")]
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
    Round    => op_nop,
    Trunc    => op_nop,
    Ceil     => op_nop,
    Floor    => op_nop,
    Neg      => op_neg,
    Abs      => op_abs,
    Swap     => op_swap,
    Remove   => op_remove,
    Dup      => op_dup,
    Jmp      => op_jmp,
    JmpZero  => op_jmp_zero,
    JmpAprxZero => op_jmp_zero,
    JmpPos      => op_jmp_pos,
    JmpFin      => op_jmp,
    JmpTos      => op_jmp_tos,
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
    instruction_limit: usize,
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

macro_rules! op_conditional_jump {
    ($name:ident, |$s:ident| $condition:expr) => {
        fn $name(&mut self) {
            let addr = self.read_address_immediate();
            let $s = &*self;
            #[allow(clippy::float_cmp)]
            if $condition {
                self.program_counter = addr;
            } else {
                self.program_counter += INSTR_PER_ADDRESS;
            }
        }
    };
}

macro_rules! op_two_operand_typed {
    ($name:ident, int: $int_method:ident, float: $float_op:tt) => {
        #[cfg(feature = "int-word")]
        op_two_operand!($name, $int_method);
        #[cfg(not(feature = "int-word"))]
        op_two_operand!($name, $float_op);
    };
}

macro_rules! op_one_operand_typed {
    ($name:ident, int: $int_method:ident, float: $float_op:tt) => {
        #[cfg(feature = "int-word")]
        op_one_operand!($name, $int_method);
        #[cfg(not(feature = "int-word"))]
        op_one_operand!($name, $float_op);
    };
}

impl Interpreter {
    /// Initialize a new interpreter with a [`Vec`] of [`Instruction`] bytes,
    /// which encode [`OpCode`]s and their immediate values.
    /// Input data can be supplied as a [`Vec`] of [`MachineWord`]s.
    /// If `program` is longer than the max value representable by `Address` it will
    /// be cut short to that length.
    /// The interpreter will halt automaticaly after `MAX_INSTRUCTIONS` instructions
    /// have been executed. Use [`Self::new_with_program_and_limit`] to set a custom limt.
    pub fn new_from_program(program: Vec<Instruction>, input: Vec<MachineWord>) -> Self {
        Self::new_with_program_and_limit(program, input, MAX_INSTRUCTIONS)
    }

    /// Functions like [`Self::new_from_program`] but allows specifying
    /// the number of instructions to execute before the interpreter halts.
    pub fn new_with_program_and_limit(
        mut program: Vec<Instruction>,
        input: Vec<MachineWord>,
        instruction_limit: usize,
    ) -> Self {
        program.truncate(Address::MAX as usize);
        Self {
            instructions: program,
            input,
            instruction_limit,
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
    ///     - Instruction limit reached
    pub fn execute(&mut self) {
        while self.execution_count < self.instruction_limit {
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

    op_conditional_jump!(op_jmp_zero, |s| s.stack.peek() == MachineWord::default());
    op_conditional_jump!(op_jmp_pos, |s| s.stack.peek() > MachineWord::default());

    op_two_operand!(op_max, max);
    op_two_operand!(op_min, min);

    op_one_operand_typed!(op_abs, int: saturating_abs, float: abs);
    op_two_operand_typed!(op_add, int: wrapping_add, float: +);
    op_two_operand_typed!(op_sub, int: wrapping_sub, float: -);
    op_two_operand_typed!(op_mul, int: wrapping_mul, float: *);
    op_one_operand_typed!(op_neg, int: saturating_neg, float: -);
}

#[cfg(feature = "int-word")]
#[allow(clippy::multiple_inherent_impl)]
impl Interpreter {
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn op_jmp_tos(&mut self) {
        let addr = self.stack.pop().saturating_abs();
        self.program_counter = (addr as usize) % self.instructions.len();
    }

    fn op_div(&mut self) {
        let b = self.stack.pop();
        let a = self.stack.pop();
        self.stack.push(a.checked_div(b).unwrap_or_default());
    }

    fn op_mod_div(&mut self) {
        let b = self.stack.pop();
        let a = self.stack.pop();
        self.stack.push(a.checked_rem(b).unwrap_or_default());
    }

    fn op_sqrt(&mut self) {
        let a = self.stack.pop();
        self.stack.push((a.unsigned_abs()).isqrt() as MachineWord);
    }
}

#[cfg(not(feature = "int-word"))]
#[allow(clippy::multiple_inherent_impl)]
impl Interpreter {
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn op_jmp_tos(&mut self) {
        let addr = self.stack.pop().abs();
        self.program_counter = (addr as usize) % self.instructions.len();
    }

    op_conditional_jump!(op_jmp_aprx_zero, |s| s.stack.peek().abs()
        <= (MachineWord::EPSILON * mw!(10)));
    op_conditional_jump!(op_jmp_fin, |s| s.stack.peek().is_finite());

    op_two_operand!(op_div, /);
    op_two_operand!(op_mod_div, %);

    fn op_sqrt(&mut self) {
        let a = self.stack.pop();
        self.stack.push((a.abs()).sqrt());
    }

    op_one_operand!(op_round, round);
    op_one_operand!(op_trunc, trunc);
    op_one_operand!(op_ceil, ceil);
    op_one_operand!(op_floor, floor);
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::util::{instructions_from_word, patch_jmp, push_jmp};

    #[test]
    fn halt_on_halt_instruction() {
        let program = vec![OpCode::Nop.into(), OpCode::Halt.into()];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(Some(HaltReason::HaltInstruction), int.halt_reason(),);
    }

    #[test]
    fn halt_on_program_end() {
        let program = vec![OpCode::Nop.into(), OpCode::Nop.into()];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
        assert_eq!(Some(HaltReason::EndOfProgram), int.halt_reason(),);
    }

    #[test]
    fn halt_on_instruction_limit() {
        let mut program = vec![OpCode::Nop.into()];
        // Constructing an infinite loop.
        let offset = push_jmp(&mut program, OpCode::Jmp);
        patch_jmp(&mut program, offset, 0);
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execution_count = MAX_INSTRUCTIONS - 10;
        int.execute();
        assert_eq!(Some(HaltReason::MaxInstructions), int.halt_reason(),);
    }

    #[test]
    fn halt_on_custom_instruction_limit() {
        const TEST_LIMIT: usize = 20;
        let mut program = vec![OpCode::Nop.into()];
        // Constructing an infinite loop.
        let offset = push_jmp(&mut program, OpCode::Jmp);
        patch_jmp(&mut program, offset, 0);
        let mut int = Interpreter::new_with_program_and_limit(program, vec![], TEST_LIMIT);
        int.execute();
        assert_eq!(
            Some(HaltReason::MaxInstructions),
            int.halt_reason(),
            "halt reason should be set to instruction limit halt"
        );
        assert_eq!(
            TEST_LIMIT, int.execution_count,
            "instruction limit halt should occur after specified number of instructions executed"
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
        assert_eq!(MachineWord::default(), int.stack.peek());
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
        assert_eq!(vec![mw!(1)], int.output());
    }

    macro_rules! test_conditional_jump {
        ($name_taken:ident, $name_skipped:ident, $val_taken:expr, $val_skipped:expr, $opcode:expr) => {
            #[test]
            fn $name_taken() {
                // If jump is not taken the data value will have been increased by one when it is output.
                let mut program: Vec<Instruction> = vec![OpCode::PushIn.into()];
                let jmp_offset = push_jmp(&mut program, $opcode);
                // jumped over if jump taken
                program.push(OpCode::PushOne.into());
                program.push(OpCode::Add.into());

                let pop_pos = program.len();
                program.extend_from_slice(&[OpCode::PopOut.into(), OpCode::Halt.into()]);
                patch_jmp(&mut program, jmp_offset, pop_pos);
                let mut int = Interpreter::new_from_program(program, vec![mw!($val_taken)]);
                int.execute();
                assert_eq!(vec![mw!($val_taken)], int.output());
            }

            #[test]
            fn $name_skipped() {
                // If jump is taken the data value will be present in the output unaltered.
                // Otherwise the output will be two.
                // This means that if `$val_skipped` is set to two this test will never fail.
                let mut program: Vec<Instruction> = vec![OpCode::PushIn.into()];
                let jmp_offset = push_jmp(&mut program, $opcode);
                // jumped over if jump taken
                program.push(OpCode::PushOne.into());
                program.push(OpCode::PushOne.into());
                program.push(OpCode::Add.into());

                let pop_pos = program.len();
                program.extend_from_slice(&[OpCode::PopOut.into(), OpCode::Halt.into()]);
                patch_jmp(&mut program, jmp_offset, pop_pos);
                let mut int = Interpreter::new_from_program(program, vec![mw!($val_skipped)]);
                int.execute();
                assert_eq!(vec![mw!(2)], int.output());
            }
        };
    }
    test_conditional_jump!(jmp_zero_taken, jmp_zero_not_taken, 0, 1, OpCode::JmpZero);
    #[cfg(not(feature = "int-word"))]
    test_conditional_jump!(
        jmp_aprx_zero_taken,
        jmp_aprx_zero_not_taken,
        mw!(9) * MachineWord::EPSILON,
        mw!(11) * MachineWord::EPSILON,
        OpCode::JmpAprxZero
    );
    test_conditional_jump!(jmp_pos_taken, jmp_pos_not_taken, 1, 0, OpCode::JmpPos);
    #[cfg(not(feature = "int-word"))]
    test_conditional_jump!(
        jmp_fin_taken,
        jmp_fin_not_taken,
        1,
        MachineWord::NAN,
        OpCode::JmpFin
    );

    #[test]
    fn opcode_jmp_tos() {
        // We measure the jump distance based on the number of `PushOne`s executed.
        let program: Vec<Instruction> = vec![
            OpCode::PushIn.into(),
            OpCode::JmpTos.into(),
            OpCode::PushOne.into(),
            OpCode::PushOne.into(),
            OpCode::PushOne.into(),
            OpCode::PushOne.into(),
            // Note that `Add` becomes effective no-op if only
            // one value is present on the stack.
            OpCode::Add.into(),
            OpCode::Add.into(),
            OpCode::Add.into(),
            OpCode::PopOut.into(),
        ];
        let mut int = Interpreter::new_from_program(program, vec![mw!(4)]);
        int.execute();
        assert_eq!(vec![mw!(2)], int.output());
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
        ($name:ident, $val_a:expr, $val_b:expr, $rslt:expr, $opcode:expr) => {
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
                assert_eq!(vec![mw!($rslt)], int.output());
            }
        };
    }
    test_two_operand_opcode!(add_opcode, 5, 6, 11, OpCode::Add);
    test_two_operand_opcode!(sub_opcode, 5, 6, 1, OpCode::Sub);
    test_two_operand_opcode!(mul_opcode, 5, 6, 30, OpCode::Mul);
    test_two_operand_opcode!(div_opcode, 5, 30, 6, OpCode::Div);
    test_two_operand_opcode!(mod_div_opcode, 7, 30, 2, OpCode::ModDiv);
    test_two_operand_opcode!(min_opcode, 5, 6, 5, OpCode::Min);
    test_two_operand_opcode!(max_opcode, 5, 6, 6, OpCode::Max);

    macro_rules! test_one_operand_opcode {
        ($name:ident, $val_a:expr, $rslt:expr, $opcode:expr) => {
            #[test]
            fn $name() {
                let program: Vec<Instruction> =
                    vec![OpCode::PushIn.into(), $opcode.into(), OpCode::PopOut.into()];
                let mut int = Interpreter::new_from_program(program, vec![mw!($val_a)]);
                int.execute();
                assert_eq!(vec![mw!($rslt)], int.output());
            }
        };
    }
    test_one_operand_opcode!(sqrt_opcode, 16, 4, OpCode::Sqrt);
    #[cfg(not(feature = "int-word"))]
    test_one_operand_opcode!(round_opcode, 5.49, 5.0, OpCode::Round);
    #[cfg(not(feature = "int-word"))]
    test_one_operand_opcode!(trunc_opcode, 5.99, 5.0, OpCode::Trunc);
    #[cfg(not(feature = "int-word"))]
    test_one_operand_opcode!(ceil_opcode, 5.01, 6.0, OpCode::Ceil);
    #[cfg(not(feature = "int-word"))]
    test_one_operand_opcode!(floor_opcode, -5.01, -6.0, OpCode::Floor);
    test_one_operand_opcode!(neg_opcode, 6, -6, OpCode::Neg);
    test_one_operand_opcode!(abs_opcode, -4, 4, OpCode::Abs);

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
        assert_eq!(vec![mw!(7)], int.output());
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
        assert_eq!(vec![mw!(7)], int.output(),);
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
        assert_eq!(vec![mw!(5), mw!(5)], int.output(),);
    }
}
