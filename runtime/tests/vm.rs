use runtime::{Instruction, Vm, VmError};

#[test]
fn addition_works() -> Result<(), VmError> {
    let mut vm = Vm::new();
    let prog = [Instruction::Push(2.0), Instruction::Push(3.0), Instruction::Add];
    let result = vm.execute(&prog)?;
    assert_eq!(result, 5.0);
    Ok(())
}

#[test]
fn multiplication_works() -> Result<(), VmError> {
    let mut vm = Vm::new();
    let prog = [Instruction::Push(4.0), Instruction::Push(6.0), Instruction::Mul];
    assert_eq!(vm.execute(&prog)?, 24.0);
    Ok(())
}

#[test]
fn division_by_zero_fails() {
    let mut vm = Vm::new();
    let prog = [Instruction::Push(1.0), Instruction::Push(0.0), Instruction::Div];
    assert_eq!(vm.execute(&prog).unwrap_err(), VmError::DivisionByZero);
}

#[test]
fn stack_underflow_detected() {
    let mut vm = Vm::new();
    let prog = [Instruction::Add];
    assert_eq!(vm.execute(&prog).unwrap_err(), VmError::StackUnderflow);
}

#[test]
fn dup_and_swap_work() -> Result<(), VmError> {
    let mut vm = Vm::new();
    let prog = [
        Instruction::Push(1.0),
        Instruction::Dup,
        Instruction::Push(2.0),
        Instruction::Swap,
        Instruction::Add,
        Instruction::Add,
    ];
    assert_eq!(vm.execute(&prog)?, 4.0);
    Ok(())
}

#[test]
fn store_and_load_work() -> Result<(), VmError> {
    let mut vm = Vm::new();
    let prog = [
        Instruction::Push(42.0),
        Instruction::Store(0),
        Instruction::Load(0),
        Instruction::Push(8.0),
        Instruction::Add,
    ];
    assert_eq!(vm.execute(&prog)?, 50.0);
    Ok(())
}
