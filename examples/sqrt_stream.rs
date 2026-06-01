//! Fixed-point square root of a float stream.
//!
//! Reads input values one at a time.
//! For each positive `x`, outputs `round(sqrt(x) * 10.0)`.
//! Values close to zero cause zero to be output.
//! The first negative value acts as a sentinel and halts.
//!
//! Disassembly as printed by this example:
//!
//! 0x0000: 03             PushIn         
//! 0x0001: 1b 0f 00       JmpPos          0x000f
//! 0x0004: 1a 09 00       JmpAprxZero     0x0009
//! 0x0007: 16             Remove         
//! 0x0008: 01             Halt           
//! 0x0009: 16             Remove         
//! 0x000a: 05             PushZero       
//! 0x000b: 06             PopOut         
//! 0x000c: 18 00 00       Jmp             0x0000
//! 0x000f: 0e             Sqrt           
//! 0x0010: 02 00 00 c8 42 PushImm         1e2
//! 0x0015: 09             Mul            
//! 0x0016: 0f             Round          
//! 0x0017: 06             PopOut         
//! 0x0018: 18 1b 00       Jmp             0x001b -> 0x0000

#![allow(clippy::doc_markdown)]
#![allow(clippy::use_debug)]

use std::time::Instant;

#[cfg(not(feature = "int-word"))]
use tardi::{
    consts::Instruction,
    disassembler::disassemble_program,
    instr::OpCode,
    interpreter::Interpreter,
    mw,
    util::{instructions_from_word, patch_jmp, push_jmp},
};

#[cfg(not(feature = "int-word"))]
fn main() {
    let mut program: Vec<Instruction> = Vec::new();

    // Read input
    let loop_start = program.len();
    program.push(OpCode::PushIn.into());

    // Branch 1: Positive value should output 100 * sqrt(x)
    let jmp_pos_patch = push_jmp(&mut program, OpCode::JmpPos);

    // Branch 2: Near zero value should output zero.
    let jmp_aprx_patch = push_jmp(&mut program, OpCode::JmpAprxZero);

    // No jumps means value is negative.
    // Sentinel was hit so halt.
    program.push(OpCode::Remove.into()); // Not stricly necessary.
    program.push(OpCode::Halt.into());

    // Start of near zero branch
    let near_zero = program.len();
    patch_jmp(&mut program, jmp_aprx_patch, near_zero);
    // Drop x
    program.push(OpCode::Remove.into());
    // Output zero
    program.push(OpCode::PushZero.into());
    program.push(OpCode::PopOut.into());
    // Jmp back to start
    let jmp_near_zero_back = push_jmp(&mut program, OpCode::Jmp);
    patch_jmp(&mut program, jmp_near_zero_back, loop_start);

    // Start of positive brach
    let positive = program.len();
    patch_jmp(&mut program, jmp_pos_patch, positive);
    // Take sqrt and multiply with 100
    program.push(OpCode::Sqrt.into());
    program.push(OpCode::PushImm.into());
    program.extend_from_slice(&instructions_from_word(mw!(100)));
    program.push(OpCode::Mul.into());
    program.push(OpCode::Round.into());
    // Output result
    program.push(OpCode::PopOut.into());
    // Jmp back to start
    let jmp_positive_back = push_jmp(&mut program, OpCode::Jmp);
    // Since jump addresses are taken modulo the program length,
    // this jump target will also end up at `loop_start`.
    let loop_start_modulo = loop_start + program.len();
    patch_jmp(&mut program, jmp_positive_back, loop_start_modulo);

    // Input is processed using pop on the intput.
    // The sentinel is placed in front so it is processed last.
    let input = vec![
        mw!(-1), // sentinel
        mw!(25),
        mw!(9),
        mw!(0),
        mw!(2),
    ];

    println!("Example: sqrt_stream.rs\nPrints fixed-point sqrt of input values.");
    println!("=== Input ===\n{input:?}\n");

    let mut s = String::new();
    // This function only returns an error variant
    // if writing to string fails, since by design
    // every sequence of bytes is a valid program.
    // `unwrap` is fine here.
    disassemble_program(&mut s, &program).unwrap();
    println!("=== Disassembly ===\n{s}");

    let mut interp = Interpreter::new_from_program(program, input);
    let start = Instant::now();
    interp.execute();
    let elapsed = start.elapsed();

    let rslt = interp.output();
    let expected = vec![mw!(500), mw!(300), mw!(0), mw!(141)];
    assert_eq!(expected, rslt, "Output should match expected values.");

    println!("=== Output ===");
    println!("{:?}", interp.output());
    println!("\nTotal Program runtime: {:?}", elapsed);
}

#[cfg(feature = "int-word")]
fn main() {
    eprintln!("This example does not work with integer machine words.")
}
