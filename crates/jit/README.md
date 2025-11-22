# SpiralX JIT Compiler

**SpiralX** is the high-performance, custom-built Just-In-Time (JIT) compiler engine for ShadowJS. SpiralX bypasses generic compiler backends to deliver raw, optimized machine code generation tailored specifically for the ShadowJS bytecode.

## Features

*   **Custom x64 Assembler**: SpiralX includes its own lightweight, lightning-fast x64 assembler, giving us complete control over instruction selection and encoding.
*   **Direct Bytecode-to-Native Translation**: Eliminates intermediate representations (IR) for faster compilation times.
*   **Optimized for ShadowJS**: The compiler is tightly coupled with the ShadowJS VM architecture, allowing for specialized optimizations that generic JITs cannot match.
*   **Micro-Optimization Capable**: Built to support fine-grained optimizations at the instruction level.

## Architecture

SpiralX takes ShadowJS bytecode chunks and compiles them directly into executable machine code in memory. It handles:

1.  **Memory Management**: Allocates executable memory pages (using `VirtualAlloc` on Windows, `mmap` on POSIX).
2.  **Code Generation**: Iterates through bytecode instructions and emits corresponding x64 machine code.
3.  **Execution**: Exposes the compiled code as a callable function pointer to the VM.

## Future Roadmap

*   **Inline Caching**: Accelerate property access.
*   **On-Stack Replacement (OSR)**: Switch from interpreter to JIT in hot loops.
*   **Deoptimization**: Bail out to the interpreter when assumptions fail.
*   **Advanced Register Allocation**: Minimize stack usage for maximum performance.

## Usage

SpiralX is enabled by default in the ShadowJS VM. It automatically detects "hot" code paths (currently all chunks) and compiles them for execution.
