use crate::opcode::OpCode;

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Constant>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
        }
    }

    pub fn write(&mut self, op: OpCode) {
        self.code.push(op);
    }

    pub fn add_constant(&mut self, value: Constant) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
