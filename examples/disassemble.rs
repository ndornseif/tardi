//! Read a fuzz artifact or raw bytecode from a file or stdin and print its disassembly.
//!
//! ## Usage
//! ```text
//! cargo run --example disassemble [--arbitrary|-a] [file]
//! cargo run --example disassemble --features int-word [--arbitrary|-a] [file]
//! ```
//! If no file is given, bytes are read from stdin.
//! By default all bytes are treated as raw bytecode with no input data.
//! Pass `--arbitrary` or `-a` to parse as an `arbitrary`-encoded fuzz artifact
//! (`Input { bytecode: Vec<Instruction>, data: Vec<MachineWord> }`).
//! ```text
//! cargo run --example disassemble --features int-word -- --arbitrary \
//!     fuzz/artifacts/fuzz_target_i32/crash-349d5a6c6ec6050d0e6651ebdb2dc2f8627e5aea
//! ```

use std::env;
use std::io::{self, Read as _};

use arbitrary::{Arbitrary, Unstructured};

use tardi::consts::{INSTRUCTION_SIZE, Instruction, MachineWord};
use tardi::disassembler::disassemble_program;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    bytecode: Vec<Instruction>,
    data: Vec<MachineWord>,
}

fn main() {
    let mut args = env::args().skip(1).peekable();

    let use_arbitrary = matches!(
        args.peek().map(String::as_str),
        Some("--arbitrary") | Some("-a")
    );
    if use_arbitrary {
        let _ = args.next();
    }

    let bytes: Vec<u8> = match args.next() {
        Some(path) => std::fs::read(&path).expect("failed to read file"),
        None => {
            let mut buf = Vec::new();
            let _ = io::stdin()
                .read_to_end(&mut buf)
                .expect("failed to read stdin");
            buf
        }
    };

    let (bytecode, data): (Vec<Instruction>, Vec<MachineWord>) = if use_arbitrary {
        match FuzzInput::arbitrary(&mut Unstructured::new(&bytes)) {
            Ok(input) => {
                eprintln!(
                    "fuzz artifact: {} byte(s) -> {} instruction(s), {} input word(s)",
                    bytes.len(),
                    input.bytecode.len(),
                    input.data.len()
                );
                (input.bytecode, input.data)
            }
            Err(e) => {
                eprintln!("error: failed to decode as arbitrary fuzz artifact: {e}");
                std::process::exit(1);
            }
        }
    } else {
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
        (instructions, vec![])
    };

    if !data.is_empty() {
        eprintln!("input data:");
        for word in &data {
            eprintln!("  {word}");
        }
    }

    let mut out = String::new();
    disassemble_program(&mut out, &bytecode).expect("disassembly failed");
    print!("{out}");
}
