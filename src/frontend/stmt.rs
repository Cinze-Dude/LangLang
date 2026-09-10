use crate::frontend::{
    ast::{Expr, Stmt, Type},
    errors::{FrontendError, ResultStmt},
    lookups::{BindingPower, STMT},
    parser,
    tokens::TokenKind,
};

impl parser::Parser {
    pub fn parse_stmt(&mut self) -> ResultStmt {
        let stmt_table = STMT.read().unwrap();

        if let Some(handler) = stmt_table.get(&self.current_token().kind) {
            handler(self)
        } else {
            let expr = self.parse_expr(BindingPower::DEFAULT)?;

            self.expect(TokenKind::SC)?;

            Ok(Box::new(Stmt::Expr(expr)))
        }
    }

    pub fn parse_block_body(&mut self) -> Result<Vec<Stmt>, FrontendError> {
        self.expect(TokenKind::SCURLY)?;

        let mut body = Vec::new();

        while self.current_token().kind != TokenKind::CCURLY {
            body.push(*self.parse_stmt()?);
        }

        self.expect(TokenKind::CCURLY)?;

        Ok(body)
    }

    pub fn parse_var(&mut self) -> ResultStmt {
        let auto = self.eat().kind == TokenKind::AUTO;
        let mut imut = false;
        let mut dynm = false;

        if self.current_token().kind == TokenKind::IMUT {
            if auto {
                return Err(FrontendError::AutoImmutable);
            }

            imut = true;
            self.eat();
        }

        if self.current_token().kind == TokenKind::DYN {
            if imut {
                return Err(FrontendError::ImutDynamic);
            }

            dynm = true;
            self.eat();
        }

        let typ = if auto {
            Type::Inferred
        } else {
            *self.parse_type(BindingPower::DEFAULT)?
        };

        let name = self.expect(TokenKind::IDENT)?.value.clone();

        match self.current_token().kind {
            TokenKind::SC => {
                if auto || imut || dynm {
                    return Err(FrontendError::MissingInitializer(name));
                }

                self.eat();

                Ok(Box::new(Stmt::Var {
                    name,
                    expr: None,
                    imut,
                    dynm,
                    typ,
                }))
            }

            TokenKind::ASSIGN => {
                self.eat();

                let result = if self.consume(TokenKind::SQRT) {
                    None
                } else {
                    Some(*self.parse_expr(BindingPower::DEFAULT)?)
                };

                self.expect(TokenKind::SC)?;

                Ok(Box::new(Stmt::Var {
                    name,
                    expr: result,
                    imut,
                    dynm,
                    typ,
                }))
            }

            _ => Err(FrontendError::ExpectedToken {
                expected: TokenKind::ASSIGN,
                found: self.current_token().kind,
            }),
        }
    }

    pub fn parse_if(&mut self) -> ResultStmt {
        self.expect(TokenKind::IF)?;

        let condition = self.parse_expr(BindingPower::DEFAULT)?;

        let then_branch = Box::new(Expr::Block(self.parse_block_body()?));

        let mut elifs = Vec::new();

        while self.consume(TokenKind::ELIF) {
            let elif_cond = self.parse_expr(BindingPower::DEFAULT)?;
            let elif_body = self.parse_block()?;

            elifs.push((elif_cond, elif_body));
        }

        let else_branch = if self.consume(TokenKind::ELSE) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Box::new(Stmt::If {
            condition,
            then_branch,
            elifs,
            else_branch,
        }))
    }

    pub fn parse_function(&mut self) -> ResultStmt {
        self.eat();

        let typ = self.parse_type(BindingPower::DEFAULT)?;

        let name = self.expect(TokenKind::IDENT)?.value.clone();

        self.expect(TokenKind::SPAREN)?;

        let mut args = Vec::new();
        let mut argtype;

        while self.current_token().kind == TokenKind::IDENT {
            argtype = self.parse_type(BindingPower::DEFAULT)?;

            args.push((self.expect(TokenKind::IDENT)?.value.clone(), *argtype));

            if self.current_token().kind == TokenKind::CPAREN {
                break; // no comma / done
            }

            self.expect(TokenKind::COMMA)?;
        }

        self.expect(TokenKind::CPAREN)?;

        let body = self.parse_block()?;

        Ok(Box::new(Stmt::Function(name, args, *typ, body)))
    }

    pub fn parse_while(&mut self) -> ResultStmt {
        let _ = self.expect(TokenKind::WHILE);
        let cond = self.parse_expr(BindingPower::DEFAULT)?;
        Ok(Box::new(Stmt::While(cond, self.parse_block()?)))
    }

    pub fn parse_for(&mut self) -> ResultStmt {
        let _ = self.expect(TokenKind::FOR);
        let iter = self.parse_expr(BindingPower::DEFAULT)?;
        Ok(Box::new(Stmt::For(iter, self.parse_block()?)))
    }
}
