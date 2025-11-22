#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Let,
    Const,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    
    // Identifiers and Literals
    Identifier(String),
    Number(f64),
    String(String),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Assign,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    
    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    SemiColon,
    Comma,
    Dot,
    
    EOF,
    Illegal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, literal: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            literal,
            line,
            column,
        }
    }
}
