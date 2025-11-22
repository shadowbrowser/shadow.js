use crate::opcode::OpCode;
use shadowjs_gc::trace::Trace;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionTemplate {
    pub name: Rc<String>,
    pub arity: usize,
    pub chunk: Chunk,
}

impl Trace for FunctionTemplate {
    fn trace(&self, visited: &mut std::collections::HashSet<usize>) {
        self.chunk.trace(visited);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(f64),
    String(Rc<String>),
    Function(Rc<FunctionTemplate>),
}

impl Trace for Constant {
    fn trace(&self, visited: &mut std::collections::HashSet<usize>) {
        match self {
            Constant::Function(f) => f.trace(visited),
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Constant>,
}

impl Trace for Chunk {
    fn trace(&self, visited: &mut std::collections::HashSet<usize>) {
        self.constants.trace(visited);
    }
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
