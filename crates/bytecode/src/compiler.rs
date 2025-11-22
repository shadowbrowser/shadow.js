use crate::opcode::OpCode;
use crate::chunk::{Chunk, Constant};
use shadowjs_ast::{Program, Statement, Expression};

pub struct BytecodeCompiler {
    chunk: Chunk,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
        }
    }

    pub fn compile(ast: &Program) -> Result<Chunk, String> {
        let mut compiler = Self::new();
        for stmt in &ast.statements {
            compiler.compile_statement(stmt)?;
        }
        Ok(compiler.chunk)
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(OpCode::Pop);
            }
            Statement::Let { name, value } | Statement::Const { name, value } => {
                self.compile_expression(value)?;
                let idx = self.chunk.add_constant(Constant::String(name.clone()));
                self.emit(OpCode::SetGlobal(idx));
                self.emit(OpCode::Pop);
            }
            Statement::Block(stmts) => {
                for stmt in stmts {
                    self.compile_statement(stmt)?;
                }
            }
            Statement::If { condition, consequence, alternative } => {
                self.compile_expression(condition)?;
                
                let jump_if_false_idx = self.emit_jump(OpCode::JumpIfFalse(0));
                
                self.compile_statement(consequence)?;
                
                let jump_idx = self.emit_jump(OpCode::Jump(0));
                
                self.patch_jump(jump_if_false_idx);
                
                if let Some(alt) = alternative {
                    self.compile_statement(alt)?;
                }
                
                self.patch_jump(jump_idx);
            }
            _ => return Err("Statement not supported yet".to_string()),
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Number(val) => {
                let idx = self.chunk.add_constant(Constant::Number(*val));
                self.emit(OpCode::Constant(idx));
            }
            Expression::String(val) => {
                let idx = self.chunk.add_constant(Constant::String(val.clone()));
                self.emit(OpCode::Constant(idx));
            }
            Expression::Identifier(name) => {
                let idx = self.chunk.add_constant(Constant::String(name.clone()));
                self.emit(OpCode::GetGlobal(idx));
            }
            Expression::Call { function, arguments } => {
                self.compile_expression(function)?;
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                self.emit(OpCode::Call(arguments.len()));
            }
            Expression::Array(elements) => {
                for elem in elements {
                    self.compile_expression(elem)?;
                }
                self.emit(OpCode::Array(elements.len()));
            }
            Expression::Object(pairs) => {
                for (key, value) in pairs {
                    let idx = self.chunk.add_constant(Constant::String(key.clone()));
                    self.emit(OpCode::Constant(idx));
                    self.compile_expression(value)?;
                }
                self.emit(OpCode::Object(pairs.len()));
            }
            Expression::Index { left, index } => {
                self.compile_expression(left)?;
                self.compile_expression(index)?;
                self.emit(OpCode::GetIndex);
            }
            Expression::Infix { left, operator, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                match operator.as_str() {
                    "+" => self.emit(OpCode::Add),
                    "-" => self.emit(OpCode::Sub),
                    "*" => self.emit(OpCode::Mul),
                    "/" => self.emit(OpCode::Div),
                    "==" => self.emit(OpCode::Equal),
                    "!=" => self.emit(OpCode::NotEqual),
                    "<" => self.emit(OpCode::LessThan),
                    ">" => self.emit(OpCode::GreaterThan),
                    _ => return Err(format!("Unknown operator: {}", operator)),
                }
            }
            _ => return Err("Expression not supported yet".to_string()),
        }
        Ok(())
    }

    fn emit(&mut self, op: OpCode) {
        self.chunk.write(op);
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit(op);
        self.chunk.code.len() - 1
    }

    fn patch_jump(&mut self, idx: usize) {
        let target = self.chunk.code.len();
        match self.chunk.code[idx] {
            OpCode::Jump(_) => {
                self.chunk.code[idx] = OpCode::Jump(target);
            }
            OpCode::JumpIfFalse(_) => {
                self.chunk.code[idx] = OpCode::JumpIfFalse(target);
            }
            _ => panic!("Cannot patch non-jump instruction"),
        }
    }
}
