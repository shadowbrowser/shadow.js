use crate::token::{Token, TokenType};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut l = Self {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
            column: 0,
        };
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
        self.column += 1;
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            if self.ch == '\n' {
                self.line += 1;
                self.column = 0;
            }
            self.read_char();
        }
    }

    fn skip_comment(&mut self) {
        while self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }
        self.skip_whitespace();
    }

    fn skip_multiline_comment(&mut self) {
        self.read_char(); // eat '*'
        self.read_char(); // eat next char
        while self.ch != '\0' {
            if self.ch == '*' && self.peek_char() == '/' {
                self.read_char(); // eat '*'
                self.read_char(); // eat '/'
                break;
            }
            self.read_char();
        }
        self.skip_whitespace();
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token_type = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    TokenType::Equal
                } else {
                    TokenType::Assign
                }
            }
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Star,
            '/' => {
                if self.peek_char() == '/' {
                    self.skip_comment();
                    return self.next_token();
                } else if self.peek_char() == '*' {
                    self.skip_multiline_comment();
                    return self.next_token();
                } else {
                    TokenType::Slash
                }
            }
            '<' => TokenType::LessThan,
            '>' => TokenType::GreaterThan,
            ';' => TokenType::SemiColon,
            '(' => TokenType::LParen,
            ')' => TokenType::RParen,
            '{' => TokenType::LBrace,
            '}' => TokenType::RBrace,
            '[' => TokenType::LBracket,
            ']' => TokenType::RBracket,
            ':' => TokenType::Colon,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '\0' => TokenType::EOF,
            '"' | '\'' => return self.read_string(self.ch),
            c if c.is_alphabetic() || c == '_' => return self.read_identifier(),
            c if c.is_digit(10) => return self.read_number(),
            _ => TokenType::Illegal,
        };

        let literal = if token_type == TokenType::EOF {
            "".to_string()
        } else {
            self.ch.to_string()
        };

        let token = Token::new(token_type, literal, self.line, self.column);
        self.read_char();
        token
    }

    fn read_identifier(&mut self) -> Token {
        let position = self.position;
        let col = self.column;
        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_char();
        }
        let literal: String = self.input[position..self.position].iter().collect();
        let token_type = match literal.as_str() {
            "let" => TokenType::Let,
            "const" => TokenType::Const,
            "function" => TokenType::Function,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            _ => TokenType::Identifier(literal.clone()),
        };
        Token::new(token_type, literal, self.line, col)
    }

    fn read_number(&mut self) -> Token {
        let position = self.position;
        let col = self.column;
        while self.ch.is_digit(10) || self.ch == '.' {
            self.read_char();
        }
        let literal: String = self.input[position..self.position].iter().collect();
        let value = literal.parse::<f64>().unwrap_or(0.0);
        Token::new(TokenType::Number(value), literal, self.line, col)
    }

    fn read_string(&mut self, quote: char) -> Token {
        let col = self.column;
        self.read_char(); // skip opening quote
        let position = self.position;
        while self.ch != quote && self.ch != '\0' {
            self.read_char();
        }
        let literal: String = self.input[position..self.position].iter().collect();
        self.read_char(); // skip closing quote
        Token::new(TokenType::String(literal.clone()), literal, self.line, col)
    }
}
