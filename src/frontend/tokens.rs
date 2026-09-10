use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    EOF,
    NUMBER,
    STRING,
    IDENT,
    RUNE,

    SPAREN,
    CPAREN,
    SBRACK,
    CBRACK,
    SCURLY,
    CCURLY,

    ASSIGN,
    EQUALS,
    TEQUAL,
    NEQUAL,

    NOT,
    AND,
    OR,
    XOR,
    LS,
    RS,

    LT,
    LE,
    GT,
    GE,

    ACCESS,
    UNTIL,
    SC,
    COLON,
    QUESTION,
    CONVERT,
    COMMA,
    POUND,
    POLE,

    INC, // Increment >+
    DEC, // Decrement >-
    RZA, // Rhizament >@
    OPP, // Opposite >--
    FIB, // Factorial >!
    PE,
    ME,
    TE,
    SE,
    MODE,
    POWE,

    PLUS,
    MINUS,
    TIMES,
    SLASH,
    MOD,
    POW,

    SIN,
    COS,
    TAN,
    ASIN,
    ACOS,
    ATAN,
    SQRT,
    FACT,

    TRUE,
    FAUX,
    LET,
    IMUT,
    CLASSDEF,
    USING,
    READ,
    NEW,
    AS,
    FOR,
    IF,
    FUNCDEF,
    ELSE,
    ELIF,
    WHILE,
    RETURN,
    TYPEDEF,
    TYPEOF,
    CASE,
    TRY,
    DO,
    AUTO,
    DYN,
    OF,
    ALIAS,

    NULL,
    INF,
    NAN,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start_pos: usize,
    pub start_line: usize,
    pub end_pos: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, value: String) -> Self {
        Self {
            kind,
            value,
            span: Span {
                start_pos: 0,
                start_line: 1,
                end_pos: 0,
                end_line: 1,
            },
        }
    }
}

pub static KEYWORDS: LazyLock<HashMap<&'static str, TokenKind>> = LazyLock::new(|| {
    HashMap::from([
        ("sin", TokenKind::SIN),
        ("cos", TokenKind::COS),
        ("tan", TokenKind::TAN),
        ("asin", TokenKind::ASIN),
        ("acos", TokenKind::ACOS),
        ("atan", TokenKind::ATAN),
        ("true", TokenKind::TRUE),
        ("false", TokenKind::FAUX),
        ("let", TokenKind::LET),
        ("imut", TokenKind::IMUT),
        ("classdef", TokenKind::CLASSDEF),
        ("using", TokenKind::USING),
        ("read", TokenKind::READ),
        ("new", TokenKind::NEW),
        ("as", TokenKind::AS),
        ("for", TokenKind::FOR),
        ("if", TokenKind::IF),
        ("funcdef", TokenKind::FUNCDEF),
        ("else", TokenKind::ELSE),
        ("elif", TokenKind::ELIF),
        ("while", TokenKind::WHILE),
        ("return", TokenKind::RETURN),
        ("typedef", TokenKind::TYPEDEF),
        ("typeof", TokenKind::TYPEOF),
        ("case", TokenKind::CASE),
        ("try", TokenKind::TRY),
        ("do", TokenKind::DO),
        ("auto", TokenKind::AUTO),
        ("dyn", TokenKind::DYN),
        ("null", TokenKind::NULL),
        ("inf", TokenKind::INF),
        ("of", TokenKind::OF),
        ("nan", TokenKind::NAN),
        ("alias", TokenKind::ALIAS),
    ])
});
