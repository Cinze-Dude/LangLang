use crate::frontend::{
    ast::{Expr, Stmt, Type},
    tokens::Token,
};

#[derive(Debug, Clone)]
pub enum FrontendError {
    UnexpectedToken(Token),
    ExpectedToken { expected: String, found: Token },
    InvalidType(String),

    InvalidExpression(String),
    InvalidSyntax(String),

    InvalidNumber(String),
    InvalidRune(String),
    InvalidOperator,
    InvalidMetadata(String),
}

pub type ResultExpr = Result<Box<Expr>, FrontendError>;
pub type ResultStmt = Result<Box<Stmt>, FrontendError>;
pub type ResultType = Result<Box<Type>, FrontendError>;
