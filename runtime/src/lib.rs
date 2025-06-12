use thiserror::Error;

pub mod token;

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
    /// Duplicate the top value on the stack.
    Dup,
    /// Swap the top two values on the stack.
    Swap,
    /// Pop a value and store it at the given memory address.
    Store(usize),
    /// Load a value from the given memory address and push it.
    Load(usize),
}

/// Minimal virtual machine for executing arithmetic instructions.
pub struct Vm {
    stack: Vec<i64>,
    memory: std::collections::HashMap<usize, i64>,
}

impl Vm {
    /// Create a new empty VM.
    pub fn new() -> Self {
        Self { stack: Vec::new(), memory: std::collections::HashMap::new() }
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
                Instruction::Dup => {
                    let val = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                    self.stack.push(val);
                }
                Instruction::Swap => {
                    if self.stack.len() < 2 {
                        return Err(VmError::StackUnderflow);
                    }
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
                Instruction::Store(addr) => {
                    let val = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    self.memory.insert(addr, val);
                }
                Instruction::Load(addr) => {
                    let val = *self.memory.get(&addr).unwrap_or(&0);
                    self.stack.push(val);
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
