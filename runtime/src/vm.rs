use crate::instruction::Instruction;

/// VM error types
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VmError {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("division by zero")]
    DivisionByZero,
    #[error("memory address {0} is invalid")]
    InvalidMemoryAddress(usize),
    #[error("program counter {0} is out of bounds")]
    InvalidProgramCounter(usize),
    #[error("stack overflow: maximum size {0} exceeded")]
    StackOverflow(usize),
    #[error("tensor operation failed: {0}")]
    TensorError(String),
    #[error("resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("hardware error: {0}")]
    HardwareError(String),
    #[error("python execution failed: {0}")]
    PythonError(String),
    #[error("ML instruction error: {0}")]
    MLInstructionError(String),
}

/// Basic VM implementation
#[derive(Debug, Clone)]
pub struct Vm {
    stack: Vec<f64>,
    memory: Vec<f64>,
    pc: usize,
    max_stack_size: usize,
    max_memory_size: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            memory: vec![0.0; 1000],
            pc: 0,
            max_stack_size: 1000,
            max_memory_size: 1000,
        }
    }

    pub fn execute(&mut self, program: &[Instruction]) -> Result<f64, VmError> {
        for instruction in program {
            self.execute_instruction(*instruction)?;
        }
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    pub fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), VmError> {
        match instruction {
            Instruction::Push(value) => {
                if self.stack.len() >= self.max_stack_size {
                    return Err(VmError::StackOverflow(self.max_stack_size));
                }
                self.stack.push(value);
            }
            Instruction::Pop => {
                self.stack.pop().ok_or(VmError::StackUnderflow)?;
            }
            Instruction::Dup => {
                let value = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                if self.stack.len() >= self.max_stack_size {
                    return Err(VmError::StackOverflow(self.max_stack_size));
                }
                self.stack.push(value);
            }
            Instruction::Swap => {
                if self.stack.len() < 2 {
                    return Err(VmError::StackUnderflow);
                }
                let len = self.stack.len();
                self.stack.swap(len - 1, len - 2);
            }
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
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                if b == 0.0 {
                    return Err(VmError::DivisionByZero);
                }
                self.stack.push(a / b);
            }
            Instruction::Store(addr) => {
                if addr >= self.max_memory_size {
                    return Err(VmError::InvalidMemoryAddress(addr));
                }
                let value = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.memory[addr] = value;
            }
            Instruction::Load(addr) => {
                if addr >= self.max_memory_size {
                    return Err(VmError::InvalidMemoryAddress(addr));
                }
                if self.stack.len() >= self.max_stack_size {
                    return Err(VmError::StackOverflow(self.max_stack_size));
                }
                self.stack.push(self.memory[addr]);
            }
            Instruction::Jump(addr) => {
                self.pc = addr;
            }
            Instruction::JumpIf(addr) => {
                let condition = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                if condition != 0.0 {
                    self.pc = addr;
                }
            }
            Instruction::Call(_addr) => {
                // Simple implementation - in a real VM this would handle function calls
                self.pc += 1;
            }
            Instruction::Return => {
                // Simple implementation - in a real VM this would return from function calls
                self.pc += 1;
            }
            Instruction::Halt => {
                // VM halts - caller should handle this
            }
            Instruction::Nop => {
                // No operation
            }
        }
        Ok(())
    }

    pub fn stack(&self) -> &[f64] {
        &self.stack
    }

    pub fn memory(&self) -> &[f64] {
        &self.memory
    }

    pub fn pc(&self) -> usize {
        self.pc
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
