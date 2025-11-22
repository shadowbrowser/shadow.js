# shadow.js

**Modern and fast JavaScript Engine written in Rust.**

> **⚠️ Warning: Experimental & In Development**
>
> This engine is currently in an early alpha stage. It is not yet ready for production use. Features are limited, and APIs may change.

## Features

shadow.js currently supports a subset of JavaScript:

*   **Variables**: `let`, `const`
*   **Data Types**: Numbers, Strings, Booleans, Arrays, Objects, Null, Undefined
*   **Functions**: Native functions (e.g., `print`) and user-defined functions (basic support)
*   **Control Flow**: `if`, `else`
*   **Comments**: Single-line (`//`) and Multi-line (`/* ... */`)
*   **Garbage Collection**: Basic reference counting (Rc) for complex types

## Installation

You can include `shadowjs` in your Rust project by adding it to your `Cargo.toml`.

```toml
[dependencies]
shadowjs-engine = { path = "crates/engine" } # Currently local path, will be on crates.io soon
```

## Usage

### Rust API

You can embed the engine directly into your Rust applications:

```rust
use shadowjs_engine::ShadowEngine;

fn main() {
    let mut engine = ShadowEngine::new();
    
    let code = r#"
        let message = "Hello from ShadowJS!";
        print(message);
        
        let data = { id: 1, value: [10, 20] };
        print(data.value[0]);
    "#;

    if let Err(e) = engine.eval(code) {
        eprintln!("Error: {}", e);
    }
}
```

### CLI

You can run JavaScript files using the CLI:

```bash
cargo run -p shadowjs -- <file.js> [--bench] [--debug]
```

*   `--bench`: Measure execution time.
*   `--debug`: Print executed opcodes.

**Example:**

```bash
cargo run -p shadowjs -- examples/complex_test.js --bench
```

## Supported Syntax Examples

```javascript
// Variables
let x = 10;
const y = 20;

// Arrays and Objects
let arr = [1, 2, 3];
let obj = { name: "Shadow", version: 1 };

// Access
print(arr[0]); // 1
print(obj.name); // Shadow

// Control Flow
if (x < y) {
    print("x is smaller");
} else {
    print("x is larger");
}

/* 
   Multi-line comments
   are also supported
*/
```

## Roadmap

*   [ ] **Garbage Collection**: Implement a proper Mark-and-Sweep GC to handle reference cycles.
*   [ ] **Loops**: Implement `while` and `for` loops.
*   [ ] **Functions**: Improve function support (closures, return values).
*   [ ] **Standard Library**: Add more built-in functions and objects (Math, Date, etc.).
*   [ ] **Error Handling**: Improve error messages and stack traces.
*   [ ] **Performance**: Optimize bytecode execution and compiler.

## License

This project is licensed under the BSD 3-Clause License - see the [LICENSE](LICENSE) file for details.
