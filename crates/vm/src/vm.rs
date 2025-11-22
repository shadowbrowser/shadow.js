use rustc_hash::FxHashMap;
use shadowjs_bytecode::{Chunk, Constant, OpCode};
use shadowjs_gc::trace::Trace;
use shadowjs_gc::{Gc, GC};
use shadowjs_jit::JitCompiler;
use shadowjs_value::Value;
use std::rc::Rc;

pub struct VM {
    stack: Vec<Value>,
    globals: FxHashMap<String, Value>,
    debug: bool,
    jit_compiler: JitCompiler,
    gc: GC,
}

impl VM {
    pub fn new() -> Self {
        let mut globals = FxHashMap::default();
        globals.insert(
            "print".to_string(),
            Value::NativeFunction(|args| {
                for arg in args {
                    print!("{} ", arg);
                }
                println!();
                Value::Undefined
            }),
        );

        Self {
            stack: Vec::with_capacity(256),
            globals,
            debug: false,
            jit_compiler: JitCompiler::new(),
            gc: GC::new(),
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn execute(&mut self, chunk: Chunk) -> Result<(), String> {
        // Try JIT first
        if let Ok(func) = self.jit_compiler.compile(&chunk) {
            if self.debug {
                println!("Executing JIT code...");
            }
            let result = func();
            self.push(Value::Number(result));
            return Ok(());
        } else {
            if self.debug {
                println!("JIT compilation failed, falling back to interpreter");
            }
        }

        let mut ip = 0;
        while ip < chunk.code.len() {
            if ip % 1000 == 0 {
                let mut roots: Vec<&dyn Trace> = Vec::new();
                for val in &self.stack {
                    roots.push(val);
                }
                for val in self.globals.values() {
                    roots.push(val);
                }
                roots.push(&chunk);

                self.gc.collect(&roots);
            }

            let op = &chunk.code[ip];
            if self.debug {
                println!("Op: {:?}", op);
            }
            ip += 1;

            match op {
                OpCode::Constant(idx) => match &chunk.constants[*idx] {
                    Constant::Number(n) => self.push(Value::Number(*n)),
                    Constant::String(s) => self.push(Value::String(s.clone())),
                    Constant::Function(f) => {
                        let func = shadowjs_value::Function {
                            name: f.name.clone(),
                            chunk: f.chunk.clone(),
                            arity: f.arity,
                        };
                        self.push(Value::Function(Gc::new(func)));
                    }
                },
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Number(a + b)),
                        (Value::String(a), Value::String(b)) => {
                            self.push(Value::String(Rc::new(format!("{}{}", a, b))))
                        }
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
                    let val = self
                        .globals
                        .get(name.as_str())
                        .ok_or(format!("Undefined variable '{}'", name))?;
                    self.push(val.clone());
                }
                OpCode::SetGlobal(idx) => {
                    let name = match &chunk.constants[*idx] {
                        Constant::String(s) => s,
                        _ => return Err("Global name must be a string".to_string()),
                    };
                    let val = self.peek(0)?.clone();
                    self.globals.insert(name.to_string(), val);
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
                    let mut map = FxHashMap::default();
                    for _ in 0..*count {
                        let value = self.pop()?;
                        let key = self.pop()?;
                        let key_str = match key {
                            Value::String(s) => s,
                            _ => return Err("Object key must be a string".to_string()),
                        };
                        map.insert(key_str.to_string(), value);
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
                            if let Some(val) = obj.get(key.as_str()) {
                                self.push(val.clone());
                            } else {
                                self.push(Value::Undefined);
                            }
                        }
                        _ => return Err("Cannot index non-array/object".to_string()),
                    }
                }
                OpCode::SetIndex => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let target = self.pop()?;
                    match target {
                        Value::Array(arr) => {
                            let idx = match index {
                                Value::Number(n) => n as usize,
                                _ => return Err("Array index must be a number".into()),
                            };
                            let mut arr = arr.borrow_mut();
                            if idx < arr.len() {
                                arr[idx] = value.clone();
                            } else if idx == arr.len() {
                                arr.push(value.clone());
                            } else {
                                return Err(
                                    "Array index out of bounds (sparse arrays not supported)"
                                        .into(),
                                );
                            }
                        }
                        Value::Object(obj) => {
                            let key = match index {
                                Value::String(s) => s,
                                _ => return Err("Object key must be a string".into()),
                            };
                            let mut obj = obj.borrow_mut();
                            obj.insert(key.to_string(), value.clone());
                        }
                        _ => return Err("Cannot set property on non-object".into()),
                    }
                    self.push(value);
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
                    }
                    self.pop()?; // Pop the condition
                }
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Undefined => self.push(Value::Undefined),
                OpCode::Null => self.push(Value::Null),
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn is_falsey(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => !*b,
            Value::Null | Value::Undefined => true,
            Value::Number(n) => *n == 0.0,
            Value::String(s) => s.is_empty(),
            _ => false,
        }
    }

    #[inline(always)]
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline(always)]
    fn pop(&mut self) -> Result<Value, &'static str> {
        self.stack.pop().ok_or("Stack underflow")
    }

    #[inline(always)]
    fn peek(&self, distance: usize) -> Result<&Value, &'static str> {
        if self.stack.len() <= distance {
            return Err("Stack underflow");
        }
        Ok(&self.stack[self.stack.len() - 1 - distance])
    }
}
