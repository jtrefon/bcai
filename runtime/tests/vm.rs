use runtime::{Instruction, Vm, VmError};

#[test]
fn addition_works() -> Result<(), VmError> {
    let mut vm = Vm::new();
    let prog = [Instruction::Push(2), Instruction::Push(3), Instruction::Add];
    let result = vm.execute(&prog)?;
    assert_eq!(result, 5);
    Ok(())
}

#[test]
fn multiplication_works() -> Result<(), VmError> {
    let mut vm = Vm::new();
    let prog = [Instruction::Push(4), Instruction::Push(6), Instruction::Mul];
    assert_eq!(vm.execute(&prog)?, 24);
    Ok(())
}

#[test]
fn division_by_zero_fails() {
    let mut vm = Vm::new();
    let prog = [Instruction::Push(1), Instruction::Push(0), Instruction::Div];
    assert_eq!(vm.execute(&prog).unwrap_err(), VmError::DivisionByZero);
}

#[test]
fn stack_underflow_detected() {
    let mut vm = Vm::new();
    let prog = [Instruction::Add];
    assert_eq!(vm.execute(&prog).unwrap_err(), VmError::StackUnderflow);
}

#[test]
fn dup_and_swap_work() {
    let mut vm = Vm::new();
    let prog = [
        Instruction::Push(1),
        Instruction::Dup,
        Instruction::Push(2),
        Instruction::Swap,
        Instruction::Add,
        Instruction::Add,
    ];
    assert_eq!(vm.execute(&prog).unwrap(), 4);
}

#[test]
fn store_and_load_work() {
    let mut vm = Vm::new();
    let prog = [
        Instruction::Push(42),
        Instruction::Store(0),
        Instruction::Load(0),
        Instruction::Push(8),
        Instruction::Add,
    ];
    assert_eq!(vm.execute(&prog).unwrap(), 50);
}
