//! Test for crash cases found by the fuzzer.

/// Both `Div` and `ModDiv` crashed if TOS was zero and an integer `MachineWord` was used.
/// Makes sense sice div by zero is well defined for floats.
///
/// ## Artifacts 
/// fuzz/artifacts/fuzz_target_i32/crash-349d5a6c6ec6050d0e6651ebdb2dc2f8627e5aea
/// fuzz/artifacts/fuzz_target_i32/crash-82eddc4252ed3758575559ad963c0118926d7725
#[cfg(all(not(feature = "long-word"), feature = "int-word", not(feature = "long-instruction"), not(feature = "long-address")))]
mod div_by_zero {
    use tardi::{interpreter::Interpreter, instr::OpCode};

    #[test]
    fn div_by_zero() {
        let program = vec![OpCode::Div.into()];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
    }

    #[test]
    fn mod_div_by_zero() {
        let program = vec![OpCode::ModDiv.into()];
        let mut int = Interpreter::new_from_program(program, vec![]);
        int.execute();
    }
}
