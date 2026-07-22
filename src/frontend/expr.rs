use core::panic;

use crate::frontend::{
    ast::{AssignOperator, BinaryOperator, Expr, Literal, PostfixOperator, PrefixOperator},
    lookups::{BP, BindingPower, LED, NUD},
    parser::Parser,
    tokens::TokenKind,
};

impl Parser {
    pub fn parse_expr(&mut self, bp: BindingPower) -> Box<Expr> {
        let t = self.current_token();
        let mut left = NUD.read().unwrap().get(&t.kind).unwrap()(self);

        while self.has_tokens() {
            let t = self.current_token();

            let Some(&next_bp) = BP.read().unwrap().get(&t.kind) else {
                break;
            };

            if next_bp <= bp {
                break;
            }

            let handler = *LED.read().unwrap().get(&t.kind).unwrap();
            left = handler(self, left, next_bp);
        }

        left
    }

    pub fn parse_primary(&mut self) -> Box<Expr> {
        let token = self.eat();

        match token.kind {
            TokenKind::NUMBER => {
                Box::new(Expr::Literal(Literal::Number(token.value.parse().unwrap())))
            }

            TokenKind::STRING => Box::new(Expr::Literal(Literal::String(token.value.clone()))),

            TokenKind::RUNE => Box::new(Expr::Literal(Literal::Rune(
                token.value.chars().nth(1).unwrap(),
            ))),

            TokenKind::TRUE => Box::new(Expr::Literal(Literal::Bool(true))),
            TokenKind::FAUX => Box::new(Expr::Literal(Literal::Bool(false))),
            TokenKind::NULL => Box::new(Expr::Literal(Literal::Null)),
            TokenKind::IDENT => Box::new(Expr::Literal(Literal::Symbol(token.value.clone()))),
            TokenKind::INF => Box::new(Expr::Literal(Literal::Infinity)),
            TokenKind::NAN => Box::new(Expr::Literal(Literal::NaN)),

            _ => panic!(
                "Parser Error: primary expression not parsed from position {} to {} on line {}",
                token.span.start_pos, token.span.end_pos, token.span.start_line,
            ),
        }
    }

    pub fn parse_binary(&mut self, expr: Box<Expr>, bp: BindingPower) -> Box<Expr> {
        let op = BinaryOperator::try_from(self.eat().kind).unwrap();
        let mut real_bp = bp;
        if op == BinaryOperator::POW {
            real_bp = BindingPower::MULTIPLY;
        }
        let right = self.parse_expr(real_bp);

        Box::new(Expr::Binary(expr, op, right))
    }

    pub fn parse_assign(&mut self, expr: Box<Expr>, bp: BindingPower) -> Box<Expr> {
        let op = AssignOperator::try_from(self.eat().kind).unwrap();
        let right = self.parse_expr(bp);
        Box::new(Expr::Assign(expr, op, right))
    }

    pub fn parse_grouping(&mut self) -> Box<Expr> {
        self.expect(TokenKind::SPAREN);

        let expr = self.parse_expr(BindingPower::DEFAULT);

        self.expect(TokenKind::CPAREN);

        expr
    }

    pub fn parse_list_expr(&mut self) -> Box<Expr> {
        self.expect(TokenKind::SBRACK);

        // []
        if self.current_token().kind == TokenKind::CBRACK {
            self.eat();
            return Box::new(Expr::Vector(Vec::new()));
        }

        let mut list = Vec::new();

        list.push(*self.parse_expr(BindingPower::DEFAULT));

        while self.current_token().kind == TokenKind::COMMA {
            self.eat();

            // allow trailing comma: [1,2,]
            if self.current_token().kind == TokenKind::CBRACK {
                break;
            }

            list.push(*self.parse_expr(BindingPower::DEFAULT));
        }

        self.expect(TokenKind::CBRACK);

        Box::new(Expr::Vector(list))
    }

    pub fn parse_until(&mut self, expr: Box<Expr>, bp: BindingPower) -> Box<Expr> {
        self.eat();
        let right = self.parse_expr(bp);
        let mut step = Box::new(Expr::Literal(Literal::Number(1.0)));
        if self.current_token().kind == TokenKind::SPAREN {
            self.eat();
            step = self.parse_expr(bp);
            self.expect(TokenKind::CPAREN);
        }
        Box::new(Expr::Range(expr, right, step))
    }

    pub fn parse_postfix(&mut self, expr: Box<Expr>, _: BindingPower) -> Box<Expr> {
        let op = PostfixOperator::try_from(self.eat().kind).unwrap();
        Box::new(Expr::Postfix(op, expr))
    }

    pub fn parse_convert(&mut self, expr: Box<Expr>, bp: BindingPower) -> Box<Expr> {
        self.eat();
        let right = self.parse_type(bp);
        Box::new(Expr::Convert(expr, *right))
    }

    pub fn parse_array_expr(&mut self, first: Expr) -> Box<Expr> {
        let mut values = vec![first];

        while self.current_token().kind == TokenKind::COMMA {
            self.eat();

            if self.current_token().kind == TokenKind::CCURLY {
                break; // trailing comma
            }

            values.push(*self.parse_expr(BindingPower::DEFAULT));
        }

        self.expect(TokenKind::CCURLY);

        Box::new(Expr::Tuple(values))
    }

    pub fn parse_map_expr(&mut self, first_key: Expr) -> Box<Expr> {
        let mut pairs = Vec::new();
        let mut key = first_key;

        loop {
            self.expect(TokenKind::COLON);

            let value = *self.parse_expr(BindingPower::DEFAULT);
            pairs.push((key.clone(), Some(value)));

            match self.current_token().kind {
                TokenKind::COMMA => {
                    self.eat();

                    if self.current_token().kind == TokenKind::CCURLY {
                        break; // trailing comma
                    }

                    key = *self.parse_expr(BindingPower::DEFAULT);
                }

                TokenKind::CCURLY => break,

                _ => {
                    self.expect(TokenKind::COMMA);
                }
            }
        }

        self.expect(TokenKind::CCURLY);

        Box::new(Expr::Map(pairs))
    }

    pub fn parse_curly_expr(&mut self) -> Box<Expr> {
        self.eat();
        let first = self.parse_expr(BindingPower::DEFAULT);

        if self.current_token().kind == TokenKind::SCURLY {
            self.parse_map_expr(*first)
        } else {
            self.parse_array_expr(*first)
        }
    }

    pub fn parse_prefix(&mut self) -> Box<Expr> {
        let op = PrefixOperator::try_from(self.eat().kind).unwrap();
        let right = self.parse_expr(BindingPower::UNARY);

        Box::new(Expr::Prefix(op, right))
    }

    pub fn parse_block(&mut self) -> Box<Expr> {
        self.eat();
        self.expect(TokenKind::SCURLY);
        let mut block = Vec::new();
        while self.current_token().kind != TokenKind::CCURLY {
            block.push(*self.parse_stmt());
        }
        self.eat();
        Box::new(Expr::Block(block))
    }

    pub fn parse_member(&mut self, left: Box<Expr>, _: BindingPower) -> Box<Expr> {
        self.eat();

        let name = self.expect(TokenKind::IDENT).value.clone();

        Box::new(Expr::Member(left, name))
    }

    pub fn parse_index(&mut self, left: Box<Expr>, _: BindingPower) -> Box<Expr> {
        self.expect(TokenKind::SBRACK);

        let index = self.parse_expr(BindingPower::DEFAULT);

        self.expect(TokenKind::CBRACK);

        Box::new(Expr::Index(left, index))
    }

    pub fn parse_call(&mut self, left: Box<Expr>, _: BindingPower) -> Box<Expr> {
        self.expect(TokenKind::SPAREN);

        let mut args = Vec::new();

        while self.current_token().kind != TokenKind::CPAREN {
            args.push(*self.parse_expr(BindingPower::DEFAULT));

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

        self.expect(TokenKind::CPAREN);

        Box::new(Expr::Call(left, args))
    }

    pub fn parse_of(&mut self, left: Box<Expr>, bp: BindingPower) -> Box<Expr> {
        self.eat();
        Box::new(Expr::Of(left, self.parse_expr(bp)))
    }
}
