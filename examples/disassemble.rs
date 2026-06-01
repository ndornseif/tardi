//! Read a program from a file or stdin and print its disassembly.
//!
//! ## Usage
//! ```text
//! cargo run --example disassemble [file]
//! cargo run --example disassemble --features int-word [file]
//! ```
//! If no file is given, bytes are read from stdin.
//! Can be used to inspect fuzzer crashes for example.
//! ```text
//! cargo run --example disassemble --features int-word \
//!     fuzz/artifacts/fuzz_target_i32/crash-349d5a6c6ec6050d0e6651ebdb2dc2f8627e5aea
//! ```

use std::env;
use std::io::{self, Read as _};

use tardi::consts::{Instruction, INSTRUCTION_SIZE};
use tardi::disassembler::disassemble_program;

fn main() {
    let bytes: Vec<u8> = match env::args().nth(1) {
        Some(path) => std::fs::read(&path).expect("failed to read file"),
        None => {
            let mut buf = Vec::new();
            let _ = io::stdin()
                .read_to_end(&mut buf)
                .expect("failed to read stdin");
            buf
        }
    };

    let trailing = bytes.len() % INSTRUCTION_SIZE;
    if trailing != 0 {
        eprintln!(
            "warning: {trailing} trailing byte(s) ignored \
             (input length not a multiple of INSTRUCTION_SIZE={INSTRUCTION_SIZE})"
        );
    }

    let instructions: Vec<Instruction> = bytes
        .chunks_exact(INSTRUCTION_SIZE)
        .map(|chunk| {
            let arr: [u8; INSTRUCTION_SIZE] = chunk.try_into().unwrap();
            Instruction::from_le_bytes(arr)
        })
        .collect();

    eprintln!(
        "{} byte(s) -> {} instruction(s)",
        bytes.len() - trailing,
        instructions.len()
    );

    let mut out = String::new();
    disassemble_program(&mut out, &instructions).expect("disassembly failed");
    print!("{out}");
}
