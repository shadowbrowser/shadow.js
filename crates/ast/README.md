# shadowjs-ast

This crate defines the Abstract Syntax Tree (AST) nodes for the ShadowJS engine.

It provides the data structures used to represent the structure of JavaScript code after parsing.

## Key Structures

*   `Program`: The root node of the AST.
*   `Statement`: Represents a statement (e.g., `VariableDeclaration`, `IfStatement`).
*   `Expression`: Represents an expression (e.g., `BinaryExpression`, `Literal`).
