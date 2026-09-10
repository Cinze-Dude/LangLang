use crate::frontend::{
    ast::Program,
    errors::FrontendError,
    lookups::{create_token_lookups, create_type_lookups},
    preprocessor::Preprocessor,
    tokens::{self, Token},
};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub index: usize,
    pub preproc: Preprocessor,
}

impl Parser {
    pub fn new(src: Vec<Token>) -> Self {
        create_token_lookups();
        create_type_lookups();
        Self {
            tokens: src,
            index: 0,
            preproc: Preprocessor::new(None, None),
        }
    }

    pub fn current_token(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub fn eat(&mut self) -> &Token {
        let idx = self.index;
        self.index += 1;
        &self.tokens[idx]
    }

    pub fn has_tokens(&self) -> bool {
        self.index < self.tokens.len() && self.current_token().kind != tokens::TokenKind::EOF
    }

    pub fn parse(&mut self) -> Result<Program, FrontendError> {
        let mut body = Vec::new();

        while self.has_tokens() {
            body.push(*self.parse_stmt()?);
        }

        self.preproc.ast = Some(Program(body));
        self.preproc.resolve_ast();

        self.preproc.ast.take().ok_or(FrontendError::Internal)
    }

    pub fn expect(&mut self, kind: tokens::TokenKind) -> Result<&Token, FrontendError> {
        let token = self.current_token();

        if token.kind != kind {
            return Err(FrontendError::UnexpectedToken(token.kind));
        }

        Ok(self.eat())
    }

    pub fn consume(&mut self, kind: tokens::TokenKind) -> bool {
        if self.current_token().kind == kind {
            self.eat();
            true
        } else {
            false
        }
    }
}
