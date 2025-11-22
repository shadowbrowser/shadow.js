use shadowjs_ast::{Expression, Program, Statement};
use shadowjs_lexer::{Lexer, Token, TokenType};

pub struct Parser {
    lexer: Lexer,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let cur_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Self {
            lexer,
            cur_token,
            peek_token,
            errors: vec![],
        }
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = vec![];

        while self.cur_token.token_type != TokenType::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.token_type {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Const => self.parse_const_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::If => self.parse_if_statement(),
            TokenType::LBrace => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_block_statement(&mut self) -> Option<Statement> {
        self.next_token(); // eat '{'
        let mut statements = vec![];

        while self.cur_token.token_type != TokenType::RBrace
            && self.cur_token.token_type != TokenType::EOF
        {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        if self.cur_token.token_type == TokenType::RBrace {
            // Idk what to do here
        }

        Some(Statement::Block(statements))
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        self.next_token(); // eat 'if'
        if self.cur_token.token_type != TokenType::LParen {
            return None;
        }
        self.next_token(); // eat '('
        let condition = self.parse_expression(0)?;

        if self.peek_token.token_type != TokenType::RParen {
            // Expect RParen
        }
        self.next_token(); // eat last token of expr
        if self.cur_token.token_type != TokenType::RParen {
            return None;
        }
        self.next_token(); // eat ')'

        let consequence = Box::new(self.parse_statement()?);

        let mut alternative = None;
        if self.peek_token.token_type == TokenType::Else {
            self.next_token(); // eat '}' or ';'
            self.next_token(); // eat 'else'
            alternative = Some(Box::new(self.parse_statement()?));
        }

        Some(Statement::If {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        // let <ident> = <expr>;
        match &self.peek_token.token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.next_token(); // eat 'let'

                if self.peek_token.token_type != TokenType::Assign {
                    return None;
                }
                self.next_token(); // eat ident
                self.next_token(); // eat '='

                let value = self.parse_expression(0)?;

                if self.peek_token.token_type == TokenType::SemiColon {
                    self.next_token();
                }

                Some(Statement::Let { name, value })
            }
            _ => None,
        }
    }

    fn parse_const_statement(&mut self) -> Option<Statement> {
        // const <ident> = <expr>;
        match &self.peek_token.token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.next_token(); // eat 'const'

                if self.peek_token.token_type != TokenType::Assign {
                    return None;
                }
                self.next_token(); // eat ident
                self.next_token(); // eat '='

                let value = self.parse_expression(0)?;

                if self.peek_token.token_type == TokenType::SemiColon {
                    self.next_token();
                }

                Some(Statement::Const { name, value })
            }
            _ => None,
        }
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token(); // eat 'return'

        let value = if self.cur_token.token_type == TokenType::SemiColon {
            None
        } else {
            Some(self.parse_expression(0)?)
        };

        if self.peek_token.token_type == TokenType::SemiColon {
            self.next_token();
        }

        Some(Statement::Return(value))
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression(0)?;

        if self.peek_token.token_type == TokenType::SemiColon {
            self.next_token();
        }

        Some(Statement::Expression(expr))
    }

    fn parse_expression(&mut self, precedence: u8) -> Option<Expression> {
        let mut left = self.parse_prefix()?;

        while self.peek_token.token_type != TokenType::SemiColon
            && precedence < self.peek_precedence()
        {
            self.next_token();
            left = self.parse_infix(left)?;
        }

        Some(left)
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let mut elements = vec![];
        if self.peek_token.token_type == TokenType::RBracket {
            self.next_token();
            return Some(Expression::Array(elements));
        }

        self.next_token();
        elements.push(self.parse_expression(0)?);

        while self.peek_token.token_type == TokenType::Comma {
            self.next_token();
            self.next_token();
            elements.push(self.parse_expression(0)?);
        }

        if self.peek_token.token_type != TokenType::RBracket {
            return None;
        }
        self.next_token();

        Some(Expression::Array(elements))
    }

    fn parse_object_literal(&mut self) -> Option<Expression> {
        let mut pairs = vec![];
        if self.peek_token.token_type == TokenType::RBrace {
            self.next_token();
            return Some(Expression::Object(pairs));
        }

        self.next_token();
        pairs.push(self.parse_object_pair()?);

        while self.peek_token.token_type == TokenType::Comma {
            self.next_token();
            self.next_token();
            pairs.push(self.parse_object_pair()?);
        }

        if self.peek_token.token_type != TokenType::RBrace {
            return None;
        }
        self.next_token();

        Some(Expression::Object(pairs))
    }

    fn parse_object_pair(&mut self) -> Option<(String, Expression)> {
        let key = match &self.cur_token.token_type {
            TokenType::Identifier(s) | TokenType::String(s) => s.clone(),
            _ => return None,
        };
        self.next_token();
        if self.cur_token.token_type != TokenType::Colon {
            return None;
        }
        self.next_token();
        let value = self.parse_expression(0)?;
        Some((key, value))
    }

    fn parse_index_expression(&mut self, left: Expression) -> Option<Expression> {
        self.next_token();
        let index = self.parse_expression(0)?;
        if self.peek_token.token_type != TokenType::RBracket {
            return None;
        }
        self.next_token();
        Some(Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
        })
    }

    fn parse_member_expression(&mut self, left: Expression) -> Option<Expression> {
        self.next_token();
        let property = match &self.cur_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => return None,
        };
        self.next_token();

        Some(Expression::Index {
            left: Box::new(left),
            index: Box::new(Expression::String(property)),
        })
    }

    fn parse_prefix(&mut self) -> Option<Expression> {
        match &self.cur_token.token_type {
            TokenType::Identifier(name) => Some(Expression::Identifier(name.clone())),
            TokenType::Number(val) => Some(Expression::Number(*val)),
            TokenType::String(val) => Some(Expression::String(val.clone())),
            TokenType::LBracket => self.parse_array_literal(),
            TokenType::LBrace => self.parse_object_literal(),
            _ => None,
        }
    }

    fn parse_infix(&mut self, left: Expression) -> Option<Expression> {
        if self.cur_token.token_type == TokenType::LParen {
            return self.parse_call_expression(left);
        }
        if self.cur_token.token_type == TokenType::LBracket {
            return self.parse_index_expression(left);
        }
        if self.cur_token.token_type == TokenType::Dot {
            return self.parse_member_expression(left);
        }

        let operator = match self.cur_token.token_type {
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Star => "*",
            TokenType::Slash => "/",
            TokenType::Equal => "==",
            TokenType::NotEqual => "!=",
            TokenType::LessThan => "<",
            TokenType::GreaterThan => ">",
            _ => return None,
        }
        .to_string();

        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;

        Some(Expression::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        self.next_token(); // eat '('
        let mut arguments = vec![];

        if self.cur_token.token_type != TokenType::RParen {
            arguments.push(self.parse_expression(0)?);
            while self.peek_token.token_type == TokenType::Comma {
                self.next_token(); // eat current arg
                self.next_token(); // eat comma
                arguments.push(self.parse_expression(0)?);
            }
        }

        if self.peek_token.token_type != TokenType::RParen {
            return None;
        }
        self.next_token(); // eat last arg or '(' if empty

        Some(Expression::Call {
            function: Box::new(function),
            arguments,
        })
    }

    fn peek_precedence(&self) -> u8 {
        match self.peek_token.token_type {
            TokenType::Dot => 6,
            TokenType::LParen | TokenType::LBracket => 5,
            TokenType::Star | TokenType::Slash => 4,
            TokenType::Plus | TokenType::Minus => 3,
            TokenType::LessThan | TokenType::GreaterThan => 2,
            TokenType::Equal | TokenType::NotEqual => 1,
            _ => 0,
        }
    }

    fn cur_precedence(&self) -> u8 {
        match self.cur_token.token_type {
            TokenType::Dot => 6,
            TokenType::LParen | TokenType::LBracket => 5,
            TokenType::Star | TokenType::Slash => 4,
            TokenType::Plus | TokenType::Minus => 3,
            TokenType::LessThan | TokenType::GreaterThan => 2,
            TokenType::Equal | TokenType::NotEqual => 1,
            _ => 0,
        }
    }
}
