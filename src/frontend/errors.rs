use crate::frontend::{
    ast::{Expr, Stmt, Type},
    tokens::TokenKind,
};

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter {
        line: usize,
        pos: usize,
        character: char,
    },
}

#[derive(Debug, Clone)]
pub enum FrontendError {
    UnexpectedToken(TokenKind),
    ExpectedToken {
        expected: TokenKind,
        found: TokenKind,
    },
    InvalidType(String),

    InvalidNumber(String),
    InvalidRune(String),
    InvalidOperator,
    InvalidMetadata(String),

    AutoImmutable,
    ImutDynamic,
    MissingInitializer(String),
}

pub type ResultExpr = Result<Box<Expr>, FrontendError>;
pub type ResultStmt = Result<Box<Stmt>, FrontendError>;
pub type ResultType = Result<Box<Type>, FrontendError>;

impl std::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrontendError::UnexpectedToken(t) => {
                write!(f, "unexpected token {:?}", t)
            }

            FrontendError::InvalidType(n) => {
                write!(f, "invalid type '{}'", n)
            }

            FrontendError::ExpectedToken {
                expected: e,
                found: g,
            } => {
                write!(f, "expected: '{:?}' but found '{:?}'", e, g)
            }

            FrontendError::InvalidNumber(n) => {
                write!(f, "invalid number '{}'", n)
            }

            FrontendError::InvalidRune(r) => {
                write!(f, "invalid rune '{}'", r)
            }

            FrontendError::InvalidOperator => {
                write!(f, "invalid operator")
            }

            FrontendError::InvalidMetadata(m) => {
                write!(f, "unknown metadata '{}'", m)
            }

            FrontendError::AutoImmutable => {
                write!(f, "auto variables cannot be immutable")
            }

            FrontendError::ImutDynamic => {
                write!(f, "dynamic variables cannot be immutable")
            }

            FrontendError::MissingInitializer(name) => {
                write!(f, "variable '{}' requires an initializer", name)
            }
        }
    }
}

impl std::error::Error for FrontendError {}
