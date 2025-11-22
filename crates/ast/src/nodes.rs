#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let { name: String, value: Expression },
    Const { name: String, value: Expression },
    Return(Option<Expression>),
    Expression(Expression),
    Block(Vec<Statement>),
    If {
        condition: Expression,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Statement>, // Block
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    Number(f64),
    String(String),
    Boolean(bool),
    Prefix {
        operator: String,
        right: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Array(Vec<Expression>),
    Object(Vec<(String, Expression)>),
    Index {
        left: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
