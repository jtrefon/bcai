use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Push(f64),
    Add,
    Sub,
    Mul,
    Div,
    Dup,
    Swap,
    Load(usize),
    Store(usize),
    Pop,
}

#[derive(Debug, Error, PartialEq)]
pub enum VmError {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("division by zero")]
    DivisionByZero,
}

#[derive(Debug, Clone)]
pub struct VmConfig {
    pub max_stack_size: usize,
    pub max_memory_size: usize,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            max_stack_size: 1024,
            max_memory_size: 1024,
        }
    }
}

#[derive(Debug)]
pub struct Vm {
    stack: Vec<f64>,
    memory: Vec<f64>,
}

impl Vm {
    pub fn new() -> Self {
        Self::with_config(VmConfig::default())
    }

    pub fn with_config(config: VmConfig) -> Self {
        Self {
            stack: Vec::with_capacity(config.max_stack_size),
            memory: vec![0.0; config.max_memory_size],
        }
    }

    pub fn execute(&mut self, program: &[Instruction]) -> Result<f64, VmError> {
        for instr in program {
            self.execute_instruction(*instr)?;
        }
        Ok(*self.stack.last().unwrap_or(&0.0))
    }

    pub fn execute_instruction(&mut self, instr: Instruction) -> Result<(), VmError> {
        match instr {
            Instruction::Push(v) => self.stack.push(v),
            Instruction::Add => self.binary(|a, b| a + b)?,
            Instruction::Sub => self.binary(|a, b| a - b)?,
            Instruction::Mul => self.binary(|a, b| a * b)?,
            Instruction::Div => {
                let b = self.pop()?;
                if b == 0.0 {
                    return Err(VmError::DivisionByZero);
                }
                let a = self.pop()?;
                self.stack.push(a / b);
            }
            Instruction::Dup => {
                let v = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                self.stack.push(v);
            }
            Instruction::Swap => {
                if self.stack.len() < 2 {
                    return Err(VmError::StackUnderflow);
                }
                let len = self.stack.len();
                self.stack.swap(len - 1, len - 2);
            }
            Instruction::Load(i) => {
                let v = *self.memory.get(i).unwrap_or(&0.0);
                self.stack.push(v);
            }
            Instruction::Store(i) => {
                let v = self.pop()?;
                if i >= self.memory.len() {
                    self.memory.resize(i + 1, 0.0);
                }
                self.memory[i] = v;
            }
            Instruction::Pop => {
                self.pop()?;
            }
        }
        Ok(())
    }

    pub fn stack(&self) -> &[f64] {
        &self.stack
    }

    fn pop(&mut self) -> Result<f64, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn binary<F>(&mut self, op: F) -> Result<(), VmError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(op(a, b));
        Ok(())
    }
}
