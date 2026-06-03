#![no_main]

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;

use tardi::interpreter::{Interpreter};
use tardi::consts::{MachineWord, Instruction};

#[derive(Debug, Arbitrary)]
struct Input {
    bytecode: Vec<Instruction>,
    data: Vec<MachineWord>,
}

fuzz_target!(|input: Input| {
    let mut interpreter = Interpreter::new(input.bytecode, input.data);
    interpreter.execute();
});

