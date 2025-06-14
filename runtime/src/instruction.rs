/// Basic VM instructions that are always available
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Instruction {
    // Stack operations
    Push(f64),
    Pop,
    Dup,
    Swap,
    
    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,
    
    // Memory operations
    Store(usize),
    Load(usize),
    
    // Control flow
    Jump(usize),
    JumpIf(usize),
    Call(usize),
    Return,
    
    // Program control
    Halt,
    Nop,
} 