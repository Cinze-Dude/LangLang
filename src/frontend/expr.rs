use crate::frontend::{
    ast::{AssignOperator, BinaryOperator, Expr, Literal, PostfixOperator, PrefixOperator},
    errors::{FrontendError, ResultExpr},
    lookups::{BP, BindingPower, LED, NUD},
    parser::Parser,
    tokens::TokenKind,
};

impl Parser {
    pub fn parse_expr(&mut self, bp: BindingPower) -> ResultExpr {
        let t = self.current_token();

        let nud = {
            let table = NUD.read().unwrap();

            *table
                .get(&t.kind)
                .ok_or_else(|| FrontendError::UnexpectedToken(t.kind))?
        };

        let mut left = nud(self)?;

        while self.has_tokens() {
            let t = self.current_token();

            let Some(&next_bp) = BP.read().unwrap().get(&t.kind) else {
                break;
            };

            if next_bp <= bp {
                break;
            }

            let led = {
                let table = LED.read().unwrap();

                *table
                    .get(&t.kind)
                    .ok_or_else(|| FrontendError::UnexpectedToken(t.kind))?
            };

            left = led(self, left, next_bp)?;
        }

        Ok(left)
    }

    pub fn parse_primary(&mut self) -> ResultExpr {
        let token = self.eat();

        let expr = match token.kind {
            TokenKind::NUMBER => Expr::Literal(Literal::Number(
                token
                    .value
                    .parse()
                    .map_err(|_| FrontendError::InvalidNumber(token.value.clone()))?,
            )),

            TokenKind::STRING => Expr::Literal(Literal::String(token.value.clone())),

            TokenKind::RUNE => Expr::Literal(Literal::Rune(
                token
                    .value
                    .chars()
                    .nth(1)
                    .ok_or_else(|| FrontendError::InvalidRune(token.value.clone()))?,
            )),

            TokenKind::TRUE => Expr::Literal(Literal::Bool(true)),
            TokenKind::FAUX => Expr::Literal(Literal::Bool(false)),
            TokenKind::NULL => Expr::Literal(Literal::Null),

            TokenKind::IDENT => Expr::Literal(Literal::Symbol(token.value.clone())),

            TokenKind::INF => Expr::Literal(Literal::Infinity),
            TokenKind::NAN => Expr::Literal(Literal::NaN),

            _ => {
                return Err(FrontendError::UnexpectedToken(token.kind));
            }
        };

        Ok(Box::new(expr))
    }

    pub fn parse_binary(&mut self, expr: Box<Expr>, bp: BindingPower) -> ResultExpr {
        let op = BinaryOperator::try_from(self.eat().kind)
            .map_err(|_| FrontendError::InvalidOperator)?;

        let real_bp = if op == BinaryOperator::POW {
            BindingPower::MULTIPLY
        } else {
            bp
        };

        let right = self.parse_expr(real_bp)?;

        Ok(Box::new(Expr::Binary(expr, op, right)))
    }

    pub fn parse_assign(&mut self, expr: Box<Expr>, bp: BindingPower) -> ResultExpr {
        let op = AssignOperator::try_from(self.eat().kind)
            .map_err(|_| FrontendError::InvalidOperator)?;

        let right = self.parse_expr(bp)?;

        Ok(Box::new(Expr::Assign(expr, op, right)))
    }

    pub fn parse_grouping(&mut self) -> ResultExpr {
        self.expect(TokenKind::SPAREN)?;

        let expr = self.parse_expr(BindingPower::DEFAULT)?;

        self.expect(TokenKind::CPAREN)?;

        Ok(expr)
    }

    pub fn parse_list_expr(&mut self) -> ResultExpr {
        self.expect(TokenKind::SBRACK)?;

        // []
        if self.current_token().kind == TokenKind::CBRACK {
            self.eat();
            return Ok(Box::new(Expr::Vector(Vec::new())));
        }

        let mut list = Vec::new();

        list.push(*self.parse_expr(BindingPower::DEFAULT)?);

        while self.current_token().kind == TokenKind::COMMA {
            self.eat();

            // allow trailing comma: [1,2,]
            if self.current_token().kind == TokenKind::CBRACK {
                break;
            }

            list.push(*self.parse_expr(BindingPower::DEFAULT)?);
        }

        self.expect(TokenKind::CBRACK)?;

        Ok(Box::new(Expr::Vector(list)))
    }

    pub fn parse_until(&mut self, expr: Box<Expr>, bp: BindingPower) -> ResultExpr {
        self.eat();

        let ae = self.consume(TokenKind::GT);

        let right = self.parse_expr(bp)?;

        let mut step = Box::new(Expr::Literal(Literal::Number(1.0)));

        if self.consume(TokenKind::COLON) {
            step = self.parse_expr(bp)?;
        }

        Ok(Box::new(Expr::Range(expr, right, step, ae)))
    }

    pub fn parse_postfix(&mut self, expr: Box<Expr>, _: BindingPower) -> ResultExpr {
        let op = PostfixOperator::try_from(self.eat().kind)
            .map_err(|_| FrontendError::InvalidOperator)?;
        Ok(Box::new(Expr::Postfix(op, expr)))
    }

    pub fn parse_convert(&mut self, expr: Box<Expr>, bp: BindingPower) -> ResultExpr {
        self.eat();

        let right = self.parse_type(bp)?;

        Ok(Box::new(Expr::Convert(expr, *right)))
    }

    pub fn parse_array_expr(&mut self, first: Expr) -> ResultExpr {
        let mut values = vec![first];

        while self.current_token().kind == TokenKind::COMMA {
            self.eat();

            if self.current_token().kind == TokenKind::CCURLY {
                break; // trailing comma
            }

            values.push(*self.parse_expr(BindingPower::DEFAULT)?);
        }

        self.expect(TokenKind::CCURLY)?;

        Ok(Box::new(Expr::Tuple(values)))
    }

    pub fn parse_map_expr(&mut self, first_key: Expr) -> ResultExpr {
        let mut pairs: Vec<(Expr, Option<Expr>)> = Vec::new();
        let mut key = first_key;

        loop {
            let value = if self.consume(TokenKind::SQRT) {
                None
            } else {
                Some(*self.parse_expr(BindingPower::DEFAULT)?)
            };
            pairs.push((key.clone(), value));

            match self.current_token().kind {
                TokenKind::COMMA => {
                    self.eat();

                    if self.current_token().kind == TokenKind::CCURLY {
                        break; // trailing comma
                    }

                    key = *self.parse_expr(BindingPower::DEFAULT)?;
                    self.expect(TokenKind::COLON)?;
                }

                TokenKind::CCURLY => break,

                _ => {
                    self.expect(TokenKind::COMMA)?;
                }
            }
        }

        self.expect(TokenKind::CCURLY)?;

        Ok(Box::new(Expr::Map(pairs)))
    }

    pub fn parse_curly_expr(&mut self) -> ResultExpr {
        self.eat();

        if self.consume(TokenKind::CCURLY) {
            return Ok(Box::new(Expr::Tuple(Vec::new())));
        }

        let first = self.parse_expr(BindingPower::DEFAULT)?;

        if self.consume(TokenKind::COLON) {
            self.parse_map_expr(*first)
        } else {
            self.parse_array_expr(*first)
        }
    }

    pub fn parse_prefix(&mut self) -> ResultExpr {
        let op = PrefixOperator::try_from(self.eat().kind)
            .map_err(|_| FrontendError::InvalidOperator)?;
        let right = self.parse_expr(BindingPower::UNARY)?;

        Ok(Box::new(Expr::Prefix(op, right)))
    }

    pub fn parse_block(&mut self) -> ResultExpr {
        self.eat();
        let _ = self.expect(TokenKind::SCURLY);
        let mut block = Vec::new();
        while self.current_token().kind != TokenKind::CCURLY {
            block.push(*self.parse_stmt()?);
        }
        self.eat();
        Ok(Box::new(Expr::Block(block)))
    }

    pub fn parse_member(&mut self, left: Box<Expr>, _: BindingPower) -> ResultExpr {
        self.eat();

        let name = self.expect(TokenKind::IDENT)?.value.clone();

        Ok(Box::new(Expr::Member(left, name)))
    }

    pub fn parse_index(&mut self, left: Box<Expr>, _: BindingPower) -> ResultExpr {
        self.expect(TokenKind::SBRACK)?;

        let index = self.parse_expr(BindingPower::DEFAULT)?;

        self.expect(TokenKind::CBRACK)?;

        Ok(Box::new(Expr::Index(left, index)))
    }

    pub fn parse_type_expr(&mut self) -> ResultExpr {
        self.expect(TokenKind::MOD)?;
        Ok(Box::new(Expr::Type(
            self.parse_type(BindingPower::DEFAULT)?,
        )))
    }

    pub fn parse_typeof(&mut self) -> ResultExpr {
        self.expect(TokenKind::TYPEOF)?;
        let right = self.parse_expr(BindingPower::UNARY)?;
        Ok(Box::new(Expr::TypeOf(right)))
    }

    pub fn parse_call(&mut self, left: Box<Expr>, _: BindingPower) -> ResultExpr {
        self.expect(TokenKind::SPAREN)?;

        let mut args = Vec::new();

        while self.current_token().kind != TokenKind::CPAREN {
            args.push(*self.parse_expr(BindingPower::DEFAULT)?);

            if self.current_token().kind == TokenKind::COMMA {
                self.eat();

                // Allow a trailing comma.
                if self.current_token().kind == TokenKind::CPAREN {
                    break;
                }
            } else {
                break;
            }
        }

        self.expect(TokenKind::CPAREN)?;

        Ok(Box::new(Expr::Call(left, args)))
    }

    pub fn parse_of(&mut self, left: Box<Expr>, bp: BindingPower) -> ResultExpr {
        self.eat();

        let right = self.parse_expr(bp)?;

        Ok(Box::new(Expr::Of(left, right)))
    }
}
