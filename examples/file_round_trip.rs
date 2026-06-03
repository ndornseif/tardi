//! Build a short program with input data, write it to a file, read it back, and print the disassembly.
//!
//! The program computes `x * x + 2` for the input value `x = 3`, so the expected
//! output when executed is `11`.
//!
//! ## Usage
//! ```text
//! cargo run --example file_round_trip
//! cargo run --example file_round_trip --features int-word
//! ```

use std::fs;

use tardi::consts::{Instruction, MachineWord};
use tardi::disassembler::disassemble_program;
use tardi::file::{read_program, write_program};
use tardi::instr::OpCode;
use tardi::mw;
use tardi::util::instructions_from_word;

fn main() {
    // Program: load x from input, duplicate it, multiply (x*x), push immediate 2, add (x*x+2),
    // emit the result, then halt.
    let mut program: Vec<Instruction> = vec![
        OpCode::PushIn.into(),
        OpCode::Dup.into(),
        OpCode::Mul.into(),
        OpCode::PushImm.into(),
    ];
    program.extend_from_slice(&instructions_from_word(mw!(2)));
    program.extend_from_slice(&[
        OpCode::Add.into(),
        OpCode::PopOut.into(),
        OpCode::Halt.into(),
    ]);

    let data: Vec<MachineWord> = vec![mw!(3)];

    let path = std::env::temp_dir().join("tardi_example.tardi");

    let file = fs::File::create(&path).expect("failed to create file");
    write_program(file, &program, &data).expect("failed to write program");
    println!(
        "wrote {} instruction(s), {} input word(s) to {}",
        program.len(),
        data.len(),
        path.display()
    );

    let file = fs::File::open(&path).expect("failed to open file");
    let (recovered_program, recovered_data) = read_program(file).expect("failed to read program");

    println!("\ninput data:");
    for word in &recovered_data {
        println!("  {word}");
    }

    println!("\ndisassembly:");
    let mut out = String::new();
    disassemble_program(&mut out, &recovered_program).expect("disassembly failed");
    print!("{out}");
}
