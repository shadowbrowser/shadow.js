use rustc_hash::FxHashMap;
use shadowjs_bytecode::Chunk;
use shadowjs_gc::trace::Trace;
use shadowjs_gc::Gc;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Rc<String>,
    pub chunk: Chunk,
    pub arity: usize,
}

impl Trace for Function {
    fn trace(&self, visited: &mut std::collections::HashSet<usize>) {
        self.chunk.trace(visited);
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(Rc<String>),
    Object(Gc<FxHashMap<String, Value>>),
    Array(Gc<Vec<Value>>),
    NativeFunction(fn(Vec<Value>) -> Value),
    Function(Gc<Function>),
    Null,
    Undefined,
}

impl Trace for Value {
    fn trace(&self, visited: &mut std::collections::HashSet<usize>) {
        match self {
            Value::Object(obj) => obj.trace(visited),
            Value::Array(arr) => arr.trace(visited),
            Value::Function(func) => func.trace(visited),
            _ => {}
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::NativeFunction(a), Value::NativeFunction(b)) => {
                // Cast to usize for comparison to avoid warning
                (*a as usize) == (*b as usize)
            }
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Object(_) => write!(f, "[object Object]"),
            Value::Array(arr) => {
                write!(f, "[")?;
                let arr = arr.borrow();
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::NativeFunction(_) => write!(f, "[native function]"),
            Value::Function(func) => write!(f, "[function {}]", func.borrow().name),
            Value::Null => write!(f, "null"),
            Value::Undefined => write!(f, "undefined"),
        }
    }
}
