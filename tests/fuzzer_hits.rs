//! Test for crash cases found by the fuzzer.

/// Both `Div` and `ModDiv` crashed if TOS was zero and an integer `MachineWord` was used.
/// Makes sense sice div by zero is well defined for floats.
///
/// ## Artifacts
/// fuzz/artifacts/fuzz_target_i32/crash-349d5a6c6ec6050d0e6651ebdb2dc2f8627e5aea
/// fuzz/artifacts/fuzz_target_i32/crash-82eddc4252ed3758575559ad963c0118926d7725
#[cfg(feature = "int-word")]
mod div_by_zero {
    use tardi::{instr::OpCode, interpreter::Interpreter};

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

/// `Sqrt` crashed when TOS was negative and integer `MachineWord` was used.
///
/// ## Artifacts
/// fuzz/artifacts/fuzz_target_i32/minimized-from-19318ae1ab74b7e3e5e66c8c46d504ee3bbe9013
#[cfg(feature = "int-word")]
mod negative_sqrt {
    use tardi::{instr::OpCode, interpreter::Interpreter, mw};

    #[test]
    fn negative_sqrt() {
        let program = vec![
            OpCode::PushIn.into(),
            OpCode::Sqrt.into(),
            OpCode::PopOut.into(),
        ];
        let mut int = Interpreter::new_from_program(program, vec![mw!(-25)]);
        int.execute();
        assert_eq!(
            vec![mw!(5)],
            int.output(),
            "sqrt of negative number should be the sqrt of its magnitude"
        );
    }
}
