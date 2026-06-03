//! Trace step by step execution of a program.
//!
//! The program to be traced and its input is read
//! from a file in the `.tardi` format supplied as arg.
//!
//! ## Usage
//! ```shell
//! cargo run --example=trace -- examples/sqrt_stream.tardi
//! ```

use std::env;
use std::fs;

use tardi::{consts::MachineWord, file::read_program, instr::OpCode, interpreter::Interpreter};

const MAX_INPUT_SHOWN: usize = 5;

fn fmt_input(input: &[MachineWord]) -> String {
    if input.is_empty() {
        return "[]".to_string();
    }
    let shown = input.len().min(MAX_INPUT_SHOWN);
    let parts: Vec<String> = input[input.len() - shown..]
        .iter()
        .rev()
        .map(|v| v.to_string())
        .collect();
    let mut s = format!("[{}]", parts.join(", "));
    if input.len() > MAX_INPUT_SHOWN {
        s.push_str(&format!(" ({} more)", input.len() - MAX_INPUT_SHOWN));
    }
    s
}

fn main() {
    let path = env::args().nth(1).expect("usage: trace <file>");
    let file = fs::File::open(&path).expect("failed to open file");
    let (recovered_program, recovered_data) = read_program(file).expect("failed to read program");

    let mut int = Interpreter::new(recovered_program, recovered_data);

    while !int.halted() && int.execution_count() < int.instruction_limit() {
        let Some(op) = int.current_opcode() else {
            println!("Reached end of program.");
            break;
        };
        let extra = match op {
            OpCode::PushImm => format!("  imm={}", int.peek_immediate()),
            OpCode::Jmp
            | OpCode::JmpZero
            | OpCode::JmpAprxZero
            | OpCode::JmpPos
            | OpCode::JmpFin => format!("  target={}", int.peek_jump_address()),
            _ => String::new(),
        };
        println!(
            "--- Step {}/{} (PC: {}) ---",
            int.execution_count() + 1,
            int.instruction_limit(),
            int.program_counter(),
        );
        println!("Instruction: {op}{extra}");
        print!("Stack:  {}", int.stack());
        print!("Output: {}", int.output_stack());
        println!("Input:  {}", fmt_input(int.input()));
        println!();
        int.dispatch();
    }
}
