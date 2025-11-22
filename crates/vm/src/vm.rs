use crate::value::Value;
use shadowjs_bytecode::{Chunk, OpCode, Constant};
use shadowjs_gc::Gc;
use std::collections::HashMap;

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    debug: bool,
}

impl VM {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert("print".to_string(), Value::NativeFunction(|args| {
            for arg in args {
                print!("{} ", arg);
            }
            println!();
            Value::Undefined
        }));

        Self {
            stack: Vec::with_capacity(256),
            globals,
            debug: false,
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn execute(&mut self, chunk: Chunk) -> Result<(), String> {
        let mut ip = 0;
        while ip < chunk.code.len() {
            let op = &chunk.code[ip];
            if self.debug {
                println!("Op: {:?}", op);
            }
            ip += 1;

            match op {
                OpCode::Constant(idx) => {
                    match &chunk.constants[*idx] {
                        Constant::Number(n) => self.push(Value::Number(*n)),
                        Constant::String(s) => self.push(Value::String(s.clone())),
                    }
                }
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Number(a + b)),
                        (Value::String(a), Value::String(b)) => self.push(Value::String(a + &b)),
                        _ => return Err("Operands must be numbers or strings".to_string()),
                    }
                }
                OpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Number(a - b)),
                        _ => return Err("Operands must be numbers".to_string()),
                    }
                }
                OpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Number(a * b)),
                        _ => return Err("Operands must be numbers".to_string()),
                    }
                }
                OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Number(a / b)),
                        _ => return Err("Operands must be numbers".to_string()),
                    }
                }
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(a == b));
                }
                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(a != b));
                }
                OpCode::LessThan => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Boolean(a < b)),
                        _ => return Err("Operands must be numbers".to_string()),
                    }
                }
                OpCode::GreaterThan => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Boolean(a > b)),
                        _ => return Err("Operands must be numbers".to_string()),
                    }
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::GetGlobal(idx) => {
                    let name = match &chunk.constants[*idx] {
                        Constant::String(s) => s,
                        _ => return Err("Global name must be a string".to_string()),
                    };
                    let val = self.globals.get(name).ok_or(format!("Undefined variable '{}'", name))?;
                    self.push(val.clone());
                }
                OpCode::SetGlobal(idx) => {
                    let name = match &chunk.constants[*idx] {
                        Constant::String(s) => s,
                        _ => return Err("Global name must be a string".to_string()),
                    };
                    let val = self.peek(0)?.clone();
                    self.globals.insert(name.clone(), val);
                }
                OpCode::Call(arg_count) => {
                    let mut args = vec![];
                    for _ in 0..*arg_count {
                        args.push(self.pop()?);
                    }
                    args.reverse();
                    let callee = self.pop()?;
                    match callee {
                        Value::NativeFunction(f) => {
                            let result = f(args);
                            self.push(result);
                        }
                        _ => return Err("Can only call functions".to_string()),
                    }
                }
                OpCode::Array(count) => {
                    let mut elements = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        elements.push(self.pop()?);
                    }
                    elements.reverse();
                    self.push(Value::Array(Gc::new(elements)));
                }
                OpCode::Object(count) => {
                    let mut map = HashMap::new();
                    for _ in 0..*count {
                        let value = self.pop()?;
                        let key = self.pop()?;
                        let key_str = match key {
                            Value::String(s) => s,
                            _ => return Err("Object key must be a string".to_string()),
                        };
                        map.insert(key_str, value);
                    }
                    self.push(Value::Object(Gc::new(map)));
                }
                OpCode::GetIndex => {
                    let index = self.pop()?;
                    let target = self.pop()?;
                    match target {
                        Value::Array(arr) => {
                            let idx = match index {
                                Value::Number(n) => n as usize,
                                _ => return Err("Array index must be a number".to_string()),
                            };
                            let arr = arr.borrow();
                            if idx < arr.len() {
                                self.push(arr[idx].clone());
                            } else {
                                self.push(Value::Undefined);
                            }
                        }
                        Value::Object(obj) => {
                            let key = match index {
                                Value::String(s) => s,
                                _ => return Err("Object key must be a string".to_string()),
                            };
                            let obj = obj.borrow();
                            if let Some(val) = obj.get(&key) {
                                self.push(val.clone());
                            } else {
                                self.push(Value::Undefined);
                            }
                        }
                        _ => return Err("Cannot index non-array/object".to_string()),
                    }
                }
                OpCode::SetIndex => {
                    // Not implemented yet in compiler, but good to have in VM
                    return Err("SetIndex not implemented".to_string());
                }
                OpCode::Jump(target) => {
                    ip = *target;
                }
                OpCode::JumpIfFalse(target) => {
                    let condition = self.peek(0)?;
                    if self.is_falsey(condition) {
                        ip = *target;
                    } else {
                        // Condition is true, fall through.
                        // But JumpIfFalse usually pops the condition?
                        // In many VMs, `if (cond)` compiles to `cond`, `JumpIfFalse`.
                        // If `cond` is used only for jump, it should be popped.
                        // My compiler emits `compile_expression(condition)`, then `JumpIfFalse`.
                        // So `condition` is on stack.
                        // If I don't pop it, stack grows.
                        // I should pop it.
                    }
                    self.pop()?; // Pop the condition
                }
                OpCode::Return => {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn is_falsey(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => !*b,
            Value::Null | Value::Undefined => true,
            Value::Number(n) => *n == 0.0,
            Value::String(s) => s.is_empty(),
            _ => false,
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, String> {
        self.stack.pop().ok_or("Stack underflow".to_string())
    }

    fn peek(&self, distance: usize) -> Result<&Value, String> {
        if self.stack.len() <= distance {
            return Err("Stack underflow".to_string());
        }
        Ok(&self.stack[self.stack.len() - 1 - distance])
    }
}
