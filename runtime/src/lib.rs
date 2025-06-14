use thiserror::Error;

pub mod evaluator;
pub mod federated;
pub mod gpu;
pub mod job_manager;
pub mod mnist;
pub mod monitoring;
pub mod network;
pub mod neural_network;
pub mod node;
pub mod p2p_service;
pub mod pouw;
pub mod security;
pub mod smart_contracts;
pub mod token;
pub mod trainer;
pub mod blockchain;
pub mod consensus_node;

pub use evaluator::*;
pub use federated::*;
pub use gpu::*;
pub use job_manager::*;
pub use mnist::*;
pub use monitoring::*;
pub use network::*;
pub use neural_network::*;
pub use node::*;
pub use p2p_service::*;
pub use pouw::*;
pub use security::*;
pub use smart_contracts::*;
pub use token::*;
pub use trainer::*;
pub use blockchain::{Blockchain, Block, Transaction, BlockchainError, BlockchainStats};
pub use consensus_node::{ConsensusNode, ConsensusConfig, MiningStats, ConsensusError};

/// Errors that can occur during VM execution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VmError {
    /// Attempted to pop from an empty stack.
    #[error("stack underflow")]
    StackUnderflow,
    /// Division by zero.
    #[error("division by zero")]
    DivisionByZero,
    /// Memory address out of bounds.
    #[error("memory address {0} is invalid")]
    InvalidMemoryAddress(usize),
    /// Program counter out of bounds.
    #[error("program counter {0} is out of bounds")]
    InvalidProgramCounter(usize),
    /// Stack overflow (too many values).
    #[error("stack overflow: maximum size {0} exceeded")]
    StackOverflow(usize),
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
    /// Halt execution and return the top of the stack.
    Halt,
}

/// Configuration for VM execution limits.
#[derive(Debug, Clone)]
pub struct VmConfig {
    pub max_stack_size: usize,
    pub max_memory_size: usize,
    pub max_instructions: usize,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self { max_stack_size: 1024, max_memory_size: 1024, max_instructions: 10000 }
    }
}

/// Minimal virtual machine for executing arithmetic instructions.
pub struct Vm {
    stack: Vec<i64>,
    memory: std::collections::HashMap<usize, i64>,
    config: VmConfig,
    instruction_count: usize,
}

impl Vm {
    /// Create a new empty VM with default configuration.
    pub fn new() -> Self {
        Self::with_config(VmConfig::default())
    }

    /// Create a new VM with custom configuration.
    pub fn with_config(config: VmConfig) -> Self {
        Self {
            stack: Vec::new(),
            memory: std::collections::HashMap::new(),
            config,
            instruction_count: 0,
        }
    }

    /// Get the current stack size.
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Get the current memory usage.
    pub fn memory_size(&self) -> usize {
        self.memory.len()
    }

    /// Get the number of instructions executed.
    pub fn instruction_count(&self) -> usize {
        self.instruction_count
    }

    /// Reset the VM to initial state.
    pub fn reset(&mut self) {
        self.stack.clear();
        self.memory.clear();
        self.instruction_count = 0;
    }

    /// Execute a single instruction.
    pub fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), VmError> {
        // Check instruction limit
        if self.instruction_count >= self.config.max_instructions {
            return Err(VmError::InvalidProgramCounter(self.instruction_count));
        }

        match *instruction {
            Instruction::Push(val) => {
                if self.stack.len() >= self.config.max_stack_size {
                    return Err(VmError::StackOverflow(self.config.max_stack_size));
                }
                self.stack.push(val);
            }
            Instruction::Add => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(a.saturating_add(b));
            }
            Instruction::Sub => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(a.saturating_sub(b));
            }
            Instruction::Mul => {
                let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(a.saturating_mul(b));
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
                if self.stack.len() >= self.config.max_stack_size {
                    return Err(VmError::StackOverflow(self.config.max_stack_size));
                }
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
                if addr >= self.config.max_memory_size {
                    return Err(VmError::InvalidMemoryAddress(addr));
                }
                let val = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.memory.insert(addr, val);
            }
            Instruction::Load(addr) => {
                if addr >= self.config.max_memory_size {
                    return Err(VmError::InvalidMemoryAddress(addr));
                }
                let val = *self.memory.get(&addr).unwrap_or(&0);
                if self.stack.len() >= self.config.max_stack_size {
                    return Err(VmError::StackOverflow(self.config.max_stack_size));
                }
                self.stack.push(val);
            }
            Instruction::Halt => {
                // Halt execution - caller should handle this
            }
        }

        self.instruction_count += 1;
        Ok(())
    }

    /// Execute a program consisting of a series of instructions.
    /// Returns the top of the stack after execution.
    pub fn execute(&mut self, program: &[Instruction]) -> Result<i64, VmError> {
        for instruction in program {
            self.execute_instruction(instruction)?;
            // Check for halt instruction
            if matches!(instruction, Instruction::Halt) {
                break;
            }
        }
        self.stack.last().cloned().ok_or(VmError::StackUnderflow)
    }

    /// Execute a program with step-by-step debugging.
    pub fn execute_debug(
        &mut self,
        program: &[Instruction],
    ) -> Result<Vec<(Instruction, i64)>, VmError> {
        let mut trace = Vec::new();

        for instruction in program {
            self.execute_instruction(instruction)?;
            let top = self.stack.last().cloned().unwrap_or(0);
            trace.push((instruction.clone(), top));

            if matches!(instruction, Instruction::Halt) {
                break;
            }
        }

        Ok(trace)
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
    fn vm_creation_and_initial_state() {
        let vm = Vm::new();
        assert_eq!(vm.stack_size(), 0);
        assert_eq!(vm.memory_size(), 0);
        assert_eq!(vm.instruction_count(), 0);
    }

    #[test]
    fn vm_with_custom_config() {
        let config = VmConfig { max_stack_size: 100, max_memory_size: 50, max_instructions: 1000 };
        let vm = Vm::with_config(config);
        assert_eq!(vm.config.max_stack_size, 100);
        assert_eq!(vm.config.max_memory_size, 50);
        assert_eq!(vm.config.max_instructions, 1000);
    }

    #[test]
    fn push_instruction() -> Result<(), VmError> {
        let mut vm = Vm::new();
        vm.execute_instruction(&Instruction::Push(42))?;
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.execute(&[])?, 42);
        Ok(())
    }

    #[test]
    fn add_instruction() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program = [Instruction::Push(10), Instruction::Push(20), Instruction::Add];
        assert_eq!(vm.execute(&program)?, 30);
        Ok(())
    }

    #[test]
    fn sub_instruction() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program = [Instruction::Push(20), Instruction::Push(10), Instruction::Sub];
        assert_eq!(vm.execute(&program)?, 10);
        Ok(())
    }

    #[test]
    fn mul_instruction() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program = [Instruction::Push(6), Instruction::Push(7), Instruction::Mul];
        assert_eq!(vm.execute(&program)?, 42);
        Ok(())
    }

    #[test]
    fn div_instruction() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program = [Instruction::Push(84), Instruction::Push(2), Instruction::Div];
        assert_eq!(vm.execute(&program)?, 42);
        Ok(())
    }

    #[test]
    fn division_by_zero_error() {
        let mut vm = Vm::new();
        let program = [Instruction::Push(10), Instruction::Push(0), Instruction::Div];
        assert_eq!(vm.execute(&program).unwrap_err(), VmError::DivisionByZero);
    }

    #[test]
    fn dup_instruction() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program = [Instruction::Push(42), Instruction::Dup, Instruction::Add];
        assert_eq!(vm.execute(&program)?, 84);
        Ok(())
    }

    #[test]
    fn swap_instruction() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program =
            [Instruction::Push(10), Instruction::Push(20), Instruction::Swap, Instruction::Sub];
        assert_eq!(vm.execute(&program)?, 10);
        Ok(())
    }

    #[test]
    fn store_and_load_instructions() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program = [
            Instruction::Push(42),
            Instruction::Store(0),
            Instruction::Load(0),
            Instruction::Push(8),
            Instruction::Add,
        ];
        assert_eq!(vm.execute(&program)?, 50);
        Ok(())
    }

    #[test]
    fn halt_instruction() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program = [
            Instruction::Push(42),
            Instruction::Halt,
            Instruction::Push(1), // Should not execute
            Instruction::Add,
        ];
        assert_eq!(vm.execute(&program)?, 42);
        assert_eq!(vm.instruction_count(), 2); // Only first two instructions executed
        Ok(())
    }

    #[test]
    fn stack_underflow_on_empty_add() {
        let mut vm = Vm::new();
        let program = [Instruction::Add];
        assert_eq!(vm.execute(&program).unwrap_err(), VmError::StackUnderflow);
    }

    #[test]
    fn stack_underflow_on_single_value_add() {
        let mut vm = Vm::new();
        let program = [Instruction::Push(5), Instruction::Add];
        assert_eq!(vm.execute(&program).unwrap_err(), VmError::StackUnderflow);
    }

    #[test]
    fn stack_underflow_on_dup_empty() {
        let mut vm = Vm::new();
        let program = [Instruction::Dup];
        assert_eq!(vm.execute(&program).unwrap_err(), VmError::StackUnderflow);
    }

    #[test]
    fn stack_underflow_on_swap_empty() {
        let mut vm = Vm::new();
        let program = [Instruction::Swap];
        assert_eq!(vm.execute(&program).unwrap_err(), VmError::StackUnderflow);
    }

    #[test]
    fn stack_underflow_on_swap_single() {
        let mut vm = Vm::new();
        let program = [Instruction::Push(1), Instruction::Swap];
        assert_eq!(vm.execute(&program).unwrap_err(), VmError::StackUnderflow);
    }

    #[test]
    fn stack_overflow_protection() {
        let config = VmConfig { max_stack_size: 2, max_memory_size: 1024, max_instructions: 1000 };
        let mut vm = Vm::with_config(config);

        // Fill stack to capacity
        vm.execute_instruction(&Instruction::Push(1)).unwrap();
        vm.execute_instruction(&Instruction::Push(2)).unwrap();

        // Next push should fail
        assert_eq!(
            vm.execute_instruction(&Instruction::Push(3)).unwrap_err(),
            VmError::StackOverflow(2)
        );
    }

    #[test]
    fn memory_address_bounds_checking() {
        let config = VmConfig { max_stack_size: 1024, max_memory_size: 10, max_instructions: 1000 };
        let mut vm = Vm::with_config(config);

        // Valid memory access
        vm.execute_instruction(&Instruction::Push(42)).unwrap();
        vm.execute_instruction(&Instruction::Store(9)).unwrap(); // Within bounds

        // Invalid memory access
        vm.execute_instruction(&Instruction::Push(42)).unwrap();
        assert_eq!(
            vm.execute_instruction(&Instruction::Store(10)).unwrap_err(),
            VmError::InvalidMemoryAddress(10)
        );

        // Invalid load
        assert_eq!(
            vm.execute_instruction(&Instruction::Load(10)).unwrap_err(),
            VmError::InvalidMemoryAddress(10)
        );
    }

    #[test]
    fn instruction_count_limit() {
        let config = VmConfig { max_stack_size: 1024, max_memory_size: 1024, max_instructions: 2 };
        let mut vm = Vm::with_config(config);

        // Execute up to limit
        vm.execute_instruction(&Instruction::Push(1)).unwrap();
        vm.execute_instruction(&Instruction::Push(2)).unwrap();

        // Next instruction should fail
        assert_eq!(
            vm.execute_instruction(&Instruction::Add).unwrap_err(),
            VmError::InvalidProgramCounter(2)
        );
    }

    #[test]
    fn integer_overflow_protection() -> Result<(), VmError> {
        let mut vm = Vm::new();

        // Test addition overflow protection (saturating)
        let program = [Instruction::Push(i64::MAX), Instruction::Push(1), Instruction::Add];
        assert_eq!(vm.execute(&program)?, i64::MAX); // Should saturate

        // Test subtraction underflow protection (saturating)
        vm.reset();
        let program = [Instruction::Push(i64::MIN), Instruction::Push(1), Instruction::Sub];
        assert_eq!(vm.execute(&program)?, i64::MIN); // Should saturate

        // Test multiplication overflow protection (saturating)
        vm.reset();
        let program = [Instruction::Push(i64::MAX), Instruction::Push(2), Instruction::Mul];
        assert_eq!(vm.execute(&program)?, i64::MAX); // Should saturate

        Ok(())
    }

    #[test]
    fn vm_reset_functionality() -> Result<(), VmError> {
        let mut vm = Vm::new();

        // Execute some operations
        vm.execute(&[Instruction::Push(42), Instruction::Store(0), Instruction::Push(10)])?;

        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.memory_size(), 1);
        assert!(vm.instruction_count() > 0);

        // Reset VM
        vm.reset();

        assert_eq!(vm.stack_size(), 0);
        assert_eq!(vm.memory_size(), 0);
        assert_eq!(vm.instruction_count(), 0);

        Ok(())
    }

    #[test]
    fn debug_execution_trace() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program =
            [Instruction::Push(10), Instruction::Push(20), Instruction::Add, Instruction::Halt];

        let trace = vm.execute_debug(&program)?;

        assert_eq!(trace.len(), 4);
        assert_eq!(trace[0], (Instruction::Push(10), 10));
        assert_eq!(trace[1], (Instruction::Push(20), 20));
        assert_eq!(trace[2], (Instruction::Add, 30));
        assert_eq!(trace[3], (Instruction::Halt, 30));

        Ok(())
    }

    #[test]
    fn load_from_uninitialized_memory() -> Result<(), VmError> {
        let mut vm = Vm::new();
        let program = [Instruction::Load(0), Instruction::Push(42), Instruction::Add];

        // Loading from uninitialized memory should return 0
        assert_eq!(vm.execute(&program)?, 42);

        Ok(())
    }

    #[test]
    fn complex_program_execution() -> Result<(), VmError> {
        let mut vm = Vm::new();

        // Calculate (a + b) * (c - d) where a=10, b=20, c=30, d=5
        let program = [
            // Calculate a + b
            Instruction::Push(10), // a
            Instruction::Push(20), // b
            Instruction::Add,      // a + b = 30
            Instruction::Store(0), // Store result
            // Calculate c - d
            Instruction::Push(30), // c
            Instruction::Push(5),  // d
            Instruction::Sub,      // c - d = 25
            // Multiply results
            Instruction::Load(0), // Load a + b
            Instruction::Swap,    // Swap to get correct order
            Instruction::Mul,     // (a + b) * (c - d) = 30 * 25 = 750
        ];

        assert_eq!(vm.execute(&program)?, 750);
        Ok(())
    }
}
