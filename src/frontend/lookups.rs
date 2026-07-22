use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use crate::frontend::{
    ast::{Expr, Stmt, Type},
    errors::{ResultExpr, ResultStmt, ResultType},
    parser::Parser,
    tokens::TokenKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingPower {
    DEFAULT,
    ASSIGN,
    OF,
    CONV,
    RANGE,
    LOGICAL,
    RELATION,
    ADDITIVE,
    MULTIPLY,
    EXPONENT,
    UNARY,
    POSTFIX,
    CALL,
    INDEX,
    MEMBER,
}

/* Handlers */

pub type NudHandler = fn(&mut Parser) -> ResultExpr;
pub type LedHandler = fn(&mut Parser, Box<Expr>, BindingPower) -> ResultExpr;
pub type StmtHandler = fn(&mut Parser) -> Box<Stmt>;

pub type TNudHandler = fn(&mut Parser) -> ResultType;
pub type TLedHandler = fn(&mut Parser, Box<Type>, BindingPower) -> ResultType;

/* Lookup Table Types */

pub type NudLookup = HashMap<TokenKind, NudHandler>;
pub type LedLookup = HashMap<TokenKind, LedHandler>;
pub type StmtLookup = HashMap<TokenKind, StmtHandler>;
pub type BPLookup = HashMap<TokenKind, BindingPower>;

pub type TNudLookup = HashMap<TokenKind, TNudHandler>;
pub type TLedLookup = HashMap<TokenKind, TLedHandler>;
pub type TBPLookup = HashMap<TokenKind, BindingPower>;

/* Lookup Tables */

pub static NUD: LazyLock<RwLock<NudLookup>> = LazyLock::new(|| RwLock::new(HashMap::new()));
pub static LED: LazyLock<RwLock<LedLookup>> = LazyLock::new(|| RwLock::new(HashMap::new()));
pub static STMT: LazyLock<RwLock<StmtLookup>> = LazyLock::new(|| RwLock::new(HashMap::new()));
pub static BP: LazyLock<RwLock<BPLookup>> = LazyLock::new(|| RwLock::new(HashMap::new()));
pub static TNUD: LazyLock<RwLock<TNudLookup>> = LazyLock::new(|| RwLock::new(HashMap::new()));
pub static TLED: LazyLock<RwLock<TLedLookup>> = LazyLock::new(|| RwLock::new(HashMap::new()));
pub static TBP: LazyLock<RwLock<TBPLookup>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn type_led(kind: TokenKind, bp: BindingPower, lf: TLedHandler) {
    TBP.write().unwrap().insert(kind, bp);
    TLED.write().unwrap().insert(kind, lf); // move here
}

pub fn type_nud(kind: TokenKind, nf: TNudHandler) {
    TNUD.write().unwrap().insert(kind, nf); // move here
}

pub fn led(kind: TokenKind, bp: BindingPower, lf: LedHandler) {
    BP.write().unwrap().insert(kind, bp);
    LED.write().unwrap().insert(kind, lf); // move here
}

pub fn nud(kind: TokenKind, nf: NudHandler) {
    NUD.write().unwrap().insert(kind, nf); // move here
}

pub fn stmt(kind: TokenKind, sf: StmtHandler) {
    BP.write().unwrap().insert(kind, BindingPower::DEFAULT);
    STMT.write().unwrap().insert(kind, sf); // move here
}

pub fn create_token_lookups() {
    led(
        TokenKind::ASSIGN,
        BindingPower::ASSIGN,
        Parser::parse_assign,
    );
    led(TokenKind::PE, BindingPower::ASSIGN, Parser::parse_assign);
    led(TokenKind::ME, BindingPower::ASSIGN, Parser::parse_assign);
    led(TokenKind::TE, BindingPower::ASSIGN, Parser::parse_assign);
    led(TokenKind::SE, BindingPower::ASSIGN, Parser::parse_assign);
    led(TokenKind::MODE, BindingPower::ASSIGN, Parser::parse_assign);
    led(TokenKind::POWE, BindingPower::ASSIGN, Parser::parse_assign);

    led(TokenKind::AND, BindingPower::LOGICAL, Parser::parse_binary);
    led(TokenKind::OR, BindingPower::LOGICAL, Parser::parse_binary);
    led(TokenKind::XOR, BindingPower::LOGICAL, Parser::parse_binary);
    led(TokenKind::RS, BindingPower::LOGICAL, Parser::parse_binary);
    led(TokenKind::LS, BindingPower::LOGICAL, Parser::parse_binary);

    led(
        TokenKind::EQUALS,
        BindingPower::RELATION,
        Parser::parse_binary,
    );
    led(
        TokenKind::NEQUAL,
        BindingPower::RELATION,
        Parser::parse_binary,
    );
    led(
        TokenKind::TEQUAL,
        BindingPower::RELATION,
        Parser::parse_binary,
    );
    led(TokenKind::LT, BindingPower::RELATION, Parser::parse_binary);
    led(TokenKind::GT, BindingPower::RELATION, Parser::parse_binary);
    led(TokenKind::LE, BindingPower::RELATION, Parser::parse_binary);
    led(TokenKind::GE, BindingPower::RELATION, Parser::parse_binary);

    led(TokenKind::POW, BindingPower::EXPONENT, Parser::parse_binary);
    led(
        TokenKind::TIMES,
        BindingPower::MULTIPLY,
        Parser::parse_binary,
    );
    led(
        TokenKind::SLASH,
        BindingPower::MULTIPLY,
        Parser::parse_binary,
    );
    led(TokenKind::MOD, BindingPower::MULTIPLY, Parser::parse_binary);
    led(
        TokenKind::PLUS,
        BindingPower::ADDITIVE,
        Parser::parse_binary,
    );
    led(
        TokenKind::MINUS,
        BindingPower::ADDITIVE,
        Parser::parse_binary,
    );
    led(TokenKind::INC, BindingPower::POSTFIX, Parser::parse_postfix);
    led(TokenKind::DEC, BindingPower::POSTFIX, Parser::parse_postfix);
    led(TokenKind::RZA, BindingPower::POSTFIX, Parser::parse_postfix);
    led(TokenKind::FIB, BindingPower::POSTFIX, Parser::parse_postfix);
    led(TokenKind::OPP, BindingPower::POSTFIX, Parser::parse_postfix);
    led(
        TokenKind::FACT,
        BindingPower::POSTFIX,
        Parser::parse_postfix,
    );

    led(
        TokenKind::CONVERT,
        BindingPower::CONV,
        Parser::parse_convert,
    );

    led(
        TokenKind::ACCESS,
        BindingPower::MEMBER,
        Parser::parse_member,
    );
    led(TokenKind::OF, BindingPower::OF, Parser::parse_of);
    led(TokenKind::UNTIL, BindingPower::RANGE, Parser::parse_until);

    nud(TokenKind::SCURLY, Parser::parse_curly_expr);
    nud(TokenKind::SBRACK, Parser::parse_list_expr);
    nud(TokenKind::SPAREN, Parser::parse_grouping);
    nud(TokenKind::DO, Parser::parse_block);

    led(TokenKind::SPAREN, BindingPower::CALL, Parser::parse_call);
    led(TokenKind::SBRACK, BindingPower::INDEX, Parser::parse_index);

    nud(TokenKind::NUMBER, Parser::parse_primary);
    nud(TokenKind::STRING, Parser::parse_primary);
    nud(TokenKind::RUNE, Parser::parse_primary);
    nud(TokenKind::TRUE, Parser::parse_primary);
    nud(TokenKind::FAUX, Parser::parse_primary);
    nud(TokenKind::NULL, Parser::parse_primary);
    nud(TokenKind::IDENT, Parser::parse_primary);
    nud(TokenKind::INF, Parser::parse_primary);
    nud(TokenKind::NAN, Parser::parse_primary);

    nud(TokenKind::MINUS, Parser::parse_prefix);
    nud(TokenKind::NOT, Parser::parse_prefix);
    nud(TokenKind::SIN, Parser::parse_prefix);
    nud(TokenKind::COS, Parser::parse_prefix);
    nud(TokenKind::TAN, Parser::parse_prefix);
    nud(TokenKind::ASIN, Parser::parse_prefix);
    nud(TokenKind::ACOS, Parser::parse_prefix);
    nud(TokenKind::ATAN, Parser::parse_prefix);
    nud(TokenKind::SQRT, Parser::parse_prefix);

    stmt(TokenKind::LET, Parser::parse_var);
    stmt(TokenKind::AUTO, Parser::parse_var);
    stmt(TokenKind::POUND, Parser::parse_metadata);
    stmt(TokenKind::IF, Parser::parse_if);
    stmt(TokenKind::WHILE, Parser::parse_while);
    stmt(TokenKind::FOR, Parser::parse_for);
}

pub fn create_type_lookups() {
    type_nud(TokenKind::IDENT, Parser::parse_symbol);
    type_nud(TokenKind::SCURLY, Parser::parse_brace_type);
    type_nud(TokenKind::SBRACK, Parser::parse_vector);
    type_led(
        TokenKind::QUESTION,
        BindingPower::POSTFIX,
        Parser::parse_option,
    );
    type_led(TokenKind::POLE, BindingPower::LOGICAL, Parser::parse_union);
}
