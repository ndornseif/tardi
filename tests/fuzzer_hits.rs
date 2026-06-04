//! Test for crash cases found by the fuzzer.

/// Both `Div` and `ModDiv` crashed if TOS was zero and an integer `MachineWord` was used.
/// Makes sense since div by zero is well defined for floats.
///
/// ## Artifacts
/// fuzz/artifacts/fuzz_target_i32/crash-349d5a6c6ec6050d0e6651ebdb2dc2f8627e5aea
/// fuzz/artifacts/fuzz_target_i32/crash-82eddc4252ed3758575559ad963c0118926d7725
#[cfg(feature = "int-word")]
mod div_by_zero {
    use tardi::{instr::OpCode, interpreter::Interpreter};

    #[test]
    fn div_by_zero() {
        let program = vec![OpCode::PushZero.into(), OpCode::Div.into()];
        let mut int = Interpreter::new(program, vec![]);
        int.execute();
    }

    #[test]
    fn mod_div_by_zero() {
        let program = vec![OpCode::PushZero.into(), OpCode::ModDiv.into()];
        let mut int = Interpreter::new(program, vec![]);
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
        let mut int = Interpreter::new(program, vec![mw!(-25)]);
        int.execute();
        assert_eq!(
            vec![mw!(5)],
            int.output(),
            "sqrt of negative number should be the sqrt of its magnitude"
        );
    }
}

/// General crashes due to integer overflow.
///
/// ## Artifacts
/// fuzz/artifacts/fuzz_target_i32/minimized-from-41dfc1f64ab83852e34b089c08f69af9391c1e73
#[cfg(feature = "int-word")]
mod int_ovfl {
    use tardi::{consts::MachineWord, instr::OpCode, interpreter::Interpreter, mw};

    #[test]
    fn mul_ovfl() {
        let program = vec![
            OpCode::PushIn.into(),
            OpCode::PushIn.into(),
            OpCode::Mul.into(),
            OpCode::PopOut.into(),
        ];
        let mut int = Interpreter::new(program, vec![MachineWord::MAX, mw!(2)]);
        int.execute();
        assert_eq!(
            vec![mw!(-2)],
            int.output(),
            "integer multiplication should be wrapping"
        );
    }

    #[test]
    fn add_ovfl() {
        let program = vec![
            OpCode::PushIn.into(),
            OpCode::PushIn.into(),
            OpCode::Add.into(),
            OpCode::PopOut.into(),
        ];
        let mut int = Interpreter::new(program, vec![MachineWord::MAX, mw!(2)]);
        int.execute();
        assert_eq!(
            vec![MachineWord::MIN + mw!(1)],
            int.output(),
            "integer addition should be wrapping"
        );
    }

    #[test]
    fn sub_unfl() {
        let program = vec![
            OpCode::PushIn.into(),
            OpCode::PushIn.into(),
            OpCode::Sub.into(),
            OpCode::PopOut.into(),
        ];
        let mut int = Interpreter::new(program, vec![mw!(2), MachineWord::MIN]);
        int.execute();
        assert_eq!(
            vec![MachineWord::MAX - mw!(1)],
            int.output(),
            "integer subtraction should be wrapping"
        );
    }
}

/// General crashes caused by trying to negate `MachineWord::MIN`.
///
/// ## Artifacts
/// fuzz/artifacts/fuzz_target_i32/minimized-from-7074034724a4fa82431d8eef481626145587944a
/// fuzz/artifacts/fuzz_target_i32/minimized-from-b2b74aa1a9a98524704428834379b4a0f8205ab7
#[cfg(feature = "int-word")]
mod int_min_negation {
    use tardi::{consts::MachineWord, instr::OpCode, interpreter::Interpreter};

    #[test]
    fn jump_tos() {
        let program = vec![OpCode::PushIn.into(), OpCode::JmpTos.into()];
        let mut int = Interpreter::new(program, vec![MachineWord::MIN]);
        int.execute();
    }

    #[test]
    fn neg() {
        let program = vec![
            OpCode::PushIn.into(),
            OpCode::Neg.into(),
            OpCode::PopOut.into(),
        ];
        let mut int = Interpreter::new(program, vec![MachineWord::MIN]);
        int.execute();
        assert_eq!(
            vec![MachineWord::MAX],
            int.output(),
            "neg of MIN should saturate to MAX"
        );
    }

    #[test]
    fn abs() {
        let program = vec![
            OpCode::PushIn.into(),
            OpCode::Abs.into(),
            OpCode::PopOut.into(),
        ];
        let mut int = Interpreter::new(program, vec![MachineWord::MIN]);
        int.execute();
        assert_eq!(
            vec![MachineWord::MAX],
            int.output(),
            "abs of MIN should saturate to MAX"
        );
    }
}
