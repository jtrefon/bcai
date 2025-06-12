use thiserror::Error;

/// Errors that can occur during VM execution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VmError {
    /// Attempted to pop from an empty stack.
    #[error("stack underflow")]
    StackUnderflow,
    /// Division by zero.
    #[error("division by zero")]
    DivisionByZero,
}

/// Simple stack-based instructions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Push a value onto the stack.
    Push(i64),
    /// Pop two values, add them, and push the result.
    Add,
    /// Pop two values, subtract, and push the result.
    Sub,
    /// Pop two values, multiply, and push the result.
    Mul,
    /// Pop two values, divide, and push the result.
    Div,
}

/// Minimal virtual machine for executing arithmetic instructions.
pub struct Vm {
    stack: Vec<i64>,
}

impl Vm {
    /// Create a new empty VM.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Execute a program consisting of a series of instructions.
    /// Returns the top of the stack after execution.
    pub fn execute(&mut self, program: &[Instruction]) -> Result<i64, VmError> {
        for inst in program {
            match *inst {
                Instruction::Push(val) => self.stack.push(val),
                Instruction::Add => {
                    let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    self.stack.push(a + b);
                }
                Instruction::Sub => {
                    let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    self.stack.push(a - b);
                }
                Instruction::Mul => {
                    let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    self.stack.push(a * b);
                }
                Instruction::Div => {
                    let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    if b == 0 {
                        return Err(VmError::DivisionByZero);
                    }
                    let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    self.stack.push(a / b);
                }
            }
        }
        self.stack.last().cloned().ok_or(VmError::StackUnderflow)
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_works() {
        let mut vm = Vm::new();
        let prog = [Instruction::Push(2), Instruction::Push(3), Instruction::Add];
        let result = vm.execute(&prog).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn multiplication_works() {
        let mut vm = Vm::new();
        let prog = [Instruction::Push(4), Instruction::Push(6), Instruction::Mul];
        assert_eq!(vm.execute(&prog).unwrap(), 24);
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
}
