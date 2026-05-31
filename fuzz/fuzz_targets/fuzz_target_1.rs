#![no_main]

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;

use tardi::interpreter::{Interpreter};

#[derive(Debug, Arbitrary)]
struct Input {
    bytecode: Vec<u8>,
    data: Vec<f32>,
}

fuzz_target!(|data: Input| {
    let mut interpreter = Interpreter::new_from_program(data.bytecode, data.data);
    interpreter.execute();
});

