use crate::frontend::{
    ast::{Expr, Stmt, Type},
    lookups::{BindingPower, STMT},
    parser,
    tokens::TokenKind,
};

impl parser::Parser {
    pub fn parse_stmt(&mut self) -> Box<Stmt> {
        let stmt_table = STMT.read().unwrap();

        if let Some(handler) = stmt_table.get(&self.current_token().kind) {
            handler(self)
        } else {
            let expr = self.parse_expr(BindingPower::DEFAULT);
            self.expect(TokenKind::SC);
            Box::new(Stmt::Expr(expr))
        }
    }

    pub fn parse_block_body(&mut self) -> Vec<Stmt> {
        self.expect(TokenKind::SCURLY);

        let mut body = Vec::new();

        while self.current_token().kind != TokenKind::CCURLY {
            body.push(*self.parse_stmt());
        }

        self.expect(TokenKind::CCURLY);

        body
    }

    pub fn parse_var(&mut self) -> Box<Stmt> {
        let auto = self.eat().kind == TokenKind::AUTO;
        let mut imut = false;
        let typ;
        let name;
        let result;
        if self.current_token().kind == TokenKind::IMUT {
            if auto {
                panic!(
                    "Parser Error: auto variable near {} on line {} cannot be immutable",
                    self.current_token().span.start_pos,
                    self.current_token().span.start_line
                )
            }
            imut = true;
            self.eat();
        }
        if auto {
            typ = Type::Inferred;
        } else {
            typ = *self.parse_type(BindingPower::DEFAULT);
        }

        name = self.expect(TokenKind::IDENT).value.clone();

        match self.current_token().kind {
            TokenKind::SC => {
                if auto || imut {
                    panic!(
                        "Parser Error: variables can only have = as an assignment operator, in position {} on line {}",
                        self.current_token().span.start_pos,
                        self.current_token().span.start_line
                    )
                }
                self.eat();
                Box::new(Stmt::Var {
                    name,
                    expr: None,
                    imut,
                    init: false,
                    typ,
                })
            }
            TokenKind::ASSIGN => {
                self.eat();
                result = *self.parse_expr(BindingPower::DEFAULT);
                self.expect(TokenKind::SC);
                Box::new(Stmt::Var {
                    name,
                    expr: Some(result),
                    imut,
                    init: true,
                    typ,
                })
            }
            _ => panic!(
                "Parser Error: variables can only have = as an assignment operator, in position {} on line {}",
                self.current_token().span.start_pos,
                self.current_token().span.start_line
            ),
        }
    }

    pub fn parse_if(&mut self) -> Box<Stmt> {
        self.expect(TokenKind::IF);

        let condition = self.parse_expr(BindingPower::DEFAULT);

        let then_branch = Box::new(Expr::Block(self.parse_block_body()));

        let mut elifs = Vec::new();

        while self.consume(TokenKind::ELIF) {
            let elif_cond = self.parse_expr(BindingPower::DEFAULT);
            let elif_body = Box::new(Expr::Block(self.parse_block_body()));

            elifs.push((elif_cond, elif_body));
        }

        let else_branch = if self.consume(TokenKind::ELSE) {
            Some(Box::new(Expr::Block(self.parse_block_body())))
        } else {
            None
        };

        Box::new(Stmt::If {
            condition,
            then_branch,
            elifs,
            else_branch,
        })
    }

    pub fn parse_while(&mut self) -> Box<Stmt> {
        self.expect(TokenKind::WHILE);
        let cond = self.parse_expr(BindingPower::DEFAULT);
        Box::new(Stmt::While(
            cond,
            Box::new(Expr::Block(self.parse_block_body())),
        ))
    }

    pub fn parse_for(&mut self) -> Box<Stmt> {
        self.expect(TokenKind::FOR);
        let iter = self.parse_expr(BindingPower::DEFAULT);
        Box::new(Stmt::For(
            iter,
            Box::new(Expr::Block(self.parse_block_body())),
        ))
    }
}
