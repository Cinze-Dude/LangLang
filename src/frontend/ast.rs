use crate::frontend::{metadata, tokens::TokenKind};
use std::{collections::HashMap, convert::TryFrom, sync::LazyLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrefixOperator {
    MINUS,
    SQRT,
    NOT,
    SIN,
    COS,
    TAN,
    ASIN,
    ACOS,
    ATAN,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    PLUS,
    MINUS,
    TIMES,
    SLASH,
    MOD,
    POW,
    AND,
    OR,
    XOR,
    EQUALS,
    NEQUAL,
    TEQUAL,
    LS,
    RS,
    LT,
    LE,
    GT,
    GE,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssignOperator {
    ASSIGN,
    PE,
    ME,
    TE,
    SE,
    POWE,
    MODE,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PostfixOperator {
    INC,
    DEC,
    OPP,
    RZA,
    FIB,
    FACT,
}

impl TryFrom<TokenKind> for AssignOperator {
    type Error = ();

    fn try_from(kind: TokenKind) -> Result<Self, Self::Error> {
        match kind {
            TokenKind::ASSIGN => Ok(Self::ASSIGN),
            TokenKind::PE => Ok(Self::PE),
            TokenKind::ME => Ok(Self::ME),
            TokenKind::TE => Ok(Self::TE),
            TokenKind::SE => Ok(Self::SE),
            TokenKind::MODE => Ok(Self::MODE),
            TokenKind::POWE => Ok(Self::POWE),
            _ => Err(()),
        }
    }
}

impl TryFrom<TokenKind> for PostfixOperator {
    type Error = ();

    fn try_from(kind: TokenKind) -> Result<Self, Self::Error> {
        match kind {
            TokenKind::INC => Ok(Self::INC),
            TokenKind::DEC => Ok(Self::DEC),
            TokenKind::RZA => Ok(Self::RZA),
            TokenKind::FIB => Ok(Self::FIB),
            TokenKind::OPP => Ok(Self::OPP),
            TokenKind::FACT => Ok(Self::FACT),
            _ => Err(()),
        }
    }
}

impl TryFrom<TokenKind> for BinaryOperator {
    type Error = ();

    fn try_from(kind: TokenKind) -> Result<Self, Self::Error> {
        match kind {
            TokenKind::PLUS => Ok(Self::PLUS),
            TokenKind::MINUS => Ok(Self::MINUS),
            TokenKind::TIMES => Ok(Self::TIMES),
            TokenKind::SLASH => Ok(Self::SLASH),
            TokenKind::MOD => Ok(Self::MOD),
            TokenKind::POW => Ok(Self::POW),

            TokenKind::AND => Ok(Self::AND),
            TokenKind::OR => Ok(Self::OR),
            TokenKind::XOR => Ok(Self::XOR),
            TokenKind::LS => Ok(Self::LS),
            TokenKind::RS => Ok(Self::RS),

            TokenKind::EQUALS => Ok(Self::EQUALS),
            TokenKind::NEQUAL => Ok(Self::NEQUAL),
            TokenKind::TEQUAL => Ok(Self::TEQUAL),
            TokenKind::LT => Ok(Self::LT),
            TokenKind::LE => Ok(Self::LE),
            TokenKind::GT => Ok(Self::GT),
            TokenKind::GE => Ok(Self::GE),

            _ => Err(()),
        }
    }
}

impl TryFrom<TokenKind> for PrefixOperator {
    type Error = ();

    fn try_from(kind: TokenKind) -> Result<Self, Self::Error> {
        match kind {
            TokenKind::MINUS => Ok(Self::MINUS),
            TokenKind::NOT => Ok(Self::NOT),
            TokenKind::SIN => Ok(Self::SIN),
            TokenKind::COS => Ok(Self::COS),
            TokenKind::TAN => Ok(Self::TAN),
            TokenKind::ASIN => Ok(Self::ASIN),
            TokenKind::ACOS => Ok(Self::ACOS),
            TokenKind::ATAN => Ok(Self::ATAN),
            TokenKind::SQRT => Ok(Self::SQRT),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Inferred,

    Symbol(String),
    String,
    Number,
    Bool,
    Rune,
    Null,
    Block,

    Tuple(Vec<Type>, usize), // fixed number of possibly different types
    Vector(Box<Type>),       // expandable, one element type

    Map(Box<Type>, Box<Type>),

    Union(Vec<Type>),
}

pub static TYPELU: LazyLock<HashMap<&'static str, Type>> = LazyLock::new(|| {
    HashMap::from([
        ("string", Type::String),
        ("number", Type::Number),
        ("bool", Type::Bool),
        ("rune", Type::Rune),
        ("null", Type::Null),
        ("Block", Type::Block),
        ("infer", Type::Inferred),
    ])
});

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Rune(char),
    Bool(bool),
    Symbol(String),
    Infinity,
    Null,
    NaN,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),

    Prefix(PrefixOperator, Box<Expr>),
    Postfix(PostfixOperator, Box<Expr>),
    Assign(Box<Expr>, AssignOperator, Box<Expr>),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Block(Vec<Stmt>),
    Convert(Box<Expr>, Type),
    Member(Box<Expr>, String),
    Range(Box<Expr>, Box<Expr>, Box<Expr>, bool), // allow-end
    Vector(Vec<Expr>),                            // Only 1 type: [1, 2, 3, 4] [number]
    Tuple(Vec<Expr>), // Many Types: {1, "cat", null, true} {} or {number || string} *the type is Type::Array*
    Map(Vec<(Expr, Option<Expr>)>), // A dictionary {number: rune}
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Of(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Box<Expr>),
    Var {
        name: String,
        expr: Option<Expr>,
        imut: bool,
        dynm: bool,
        init: bool,
        typ: Type,
    },
    Metadata(metadata::Metadata, metadata::Status),
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>, // Expr::Block
        elifs: Vec<(Box<Expr>, Box<Expr>)>,
        else_branch: Option<Box<Expr>>,
    },
    While(Box<Expr>, Box<Expr>),
    For(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program(pub Vec<Stmt>);
