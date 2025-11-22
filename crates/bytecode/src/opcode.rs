#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    Constant(usize),
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    Pop,
    GetGlobal(usize), // Index of name in constants
    SetGlobal(usize), // Index of name in constants
    Call(usize),      // Number of arguments
    Array(usize),     // Number of elements
    Object(usize),    // Number of pairs
    GetIndex,
    SetIndex,
    Jump(usize),      // Relative jump
    JumpIfFalse(usize), // Relative jump if false
    Return,
}
