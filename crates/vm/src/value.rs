use shadowjs_gc::Gc;
use shadowjs_bytecode::Chunk;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub chunk: Chunk,
    pub arity: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Object(Gc<HashMap<String, Value>>),
    Array(Gc<Vec<Value>>),
    NativeFunction(fn(Vec<Value>) -> Value),
    Function(Gc<Function>),
    Null,
    Undefined,
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
