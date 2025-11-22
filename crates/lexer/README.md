# shadowjs-lexer

The Lexical Analyzer for ShadowJS.

This crate is responsible for taking raw source code strings and converting them into a stream of `Token`s. It handles:
*   Identifier resolution
*   Keyword matching
*   Literal parsing (numbers, strings)
*   Comment skipping
