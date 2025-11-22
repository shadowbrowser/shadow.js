# shadowjs-bytecode

This crate handles the bytecode generation and representation for ShadowJS.

## Responsibilities

*   **OpCode Definition**: Defines the instruction set for the ShadowJS VM.
*   **Compiler**: Transforms the AST (from `shadowjs-ast`) into a linear sequence of bytecode instructions (Chunks).
*   **Chunk**: A container for bytecode and constants.
