use std::{collections::HashMap, sync::LazyLock};

use crate::frontend::{
    ast::Stmt,
    parser::Parser,
    tokens::TokenKind::{self},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Deactivation,
    Activation,
    Persist,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Metadata {
    DEPRECATED,
    WINDOWS,
    UNIX,
    NOMANGLE,
    CONSTRUCT,
    EXPORT,
}

pub static METADATA: LazyLock<HashMap<&'static str, Metadata>> = LazyLock::new(|| {
    HashMap::from([
        ("DEPRECATED", Metadata::DEPRECATED),
        ("WINDOWS", Metadata::WINDOWS),
        ("UNIX", Metadata::UNIX),
        ("NO_MANGLE", Metadata::NOMANGLE),
        ("CONSTRUCT", Metadata::CONSTRUCT),
        ("EXPORT", Metadata::EXPORT),
    ])
});

impl Parser {
    pub fn parse_metadata(&mut self) -> Box<Stmt> {
        self.expect(TokenKind::POUND);
        let mut status = Status::Activation;
        if self.current_token().kind == TokenKind::NOT {
            status = Status::Deactivation;
            self.eat();
        } else if self.current_token().kind == TokenKind::FACT {
            status = Status::Persist;
            self.eat();
        }

        let token = self.expect(TokenKind::IDENT);

        let metadata = *METADATA.get(token.value.as_str()).unwrap_or_else(|| {
            panic!(
                "Parser Error: unknown metadata '{}' at line {}, position {}",
                token.value, token.span.start_line, token.span.start_pos,
            )
        });

        Box::new(Stmt::Metadata(metadata, status))
    }
}
