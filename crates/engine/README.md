# shadowjs-engine

The high-level API for the ShadowJS engine.

This crate acts as the entry point for embedding ShadowJS into Rust applications. It orchestrates the parsing, compilation, and execution phases.

## Usage

```rust
use shadowjs_engine::ShadowEngine;

let mut engine = ShadowEngine::new();
engine.eval("print('Hello World');").unwrap();
```
