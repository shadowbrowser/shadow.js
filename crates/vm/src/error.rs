use std::fmt;

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow,
    UndefinedVariable(String),
    TypeError(String),
    UnknownOperator(String),
    Custom(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeError::StackUnderflow => write!(f, "Stack underflow"),
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable '{}'", name),
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::UnknownOperator(op) => write!(f, "Unknown operator: {}", op),
            RuntimeError::Custom(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}
