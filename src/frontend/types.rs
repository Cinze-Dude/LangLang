use crate::frontend::{
    ast::{TYPELU, Type},
    lookups::{BindingPower, TBP, TLED, TNUD},
    parser::Parser,
    tokens::TokenKind,
};

impl Parser {
    pub fn parse_type(&mut self, bp: BindingPower) -> Box<Type> {
        let t = self.current_token();

        let mut left = TNUD.read().unwrap().get(&t.kind).unwrap()(self);

        loop {
            if !self.has_tokens() {
                break;
            }

            let t = self.current_token();

            let tbp = {
                let bp_table = TBP.read().unwrap();
                match bp_table.get(&t.kind) {
                    Some(token_bp) if *token_bp > bp => *token_bp,
                    _ => break,
                }
            };

            let handler = {
                let led_table = TLED.read().unwrap();
                match led_table.get(&t.kind) {
                    Some(handler) => *handler,
                    None => panic!(
                        "Parser Error: no type LED handler for {:?} at line {}, position {}",
                        t.kind, t.span.start_line, t.span.start_pos,
                    ),
                }
            };

            left = handler(self, left, tbp);
        }

        left
    }

    pub fn parse_symbol(&mut self) -> Box<Type> {
        let token = self.eat();

        if let Some(ty) = TYPELU.get(token.value.as_str()) {
            Box::new(ty.clone())
        } else {
            Box::new(Type::Symbol(token.value.clone()))
        }
    }

    pub fn parse_tuple_type(&mut self, first: Box<Type>) -> Box<Type> {
        let mut types = vec![*first];

        while self.current_token().kind == TokenKind::POLE {
            self.eat();
            types.push(*self.parse_type(BindingPower::DEFAULT));
        }

        self.expect(TokenKind::SC);

        let length = self
            .expect(TokenKind::NUMBER)
            .value
            .parse()
            .expect("Tuple length must be numerical");

        self.expect(TokenKind::CCURLY);

        Box::new(Type::Tuple(types, length))
    }

    pub fn parse_map_type(&mut self, first: Box<Type>) -> Box<Type> {
        let key = first;
        self.expect(TokenKind::COLON);
        let val = self.parse_type(BindingPower::DEFAULT);
        self.expect(TokenKind::CCURLY);
        Box::new(Type::Map(key, val))
    }

    pub fn parse_brace_type(&mut self) -> Box<Type> {
        self.expect(TokenKind::SCURLY);

        let first = self.parse_type(BindingPower::DEFAULT);

        if self.current_token().kind == TokenKind::COLON {
            self.parse_map_type(first)
        } else {
            self.parse_tuple_type(first)
        }
    }

    pub fn parse_vector(&mut self) -> Box<Type> {
        self.expect(TokenKind::SBRACK);

        let ty = self.parse_type(BindingPower::DEFAULT);

        self.expect(TokenKind::CBRACK);

        Box::new(Type::Vector(ty))
    }

    pub fn parse_option(&mut self, ty: Box<Type>, bp: BindingPower) -> Box<Type> {
        self.parse_type(bp);
        self.expect(TokenKind::QUESTION);
        Box::new(Type::Union(vec![*ty, Type::Null]))
    }

    pub fn parse_union(&mut self, left: Box<Type>, bp: BindingPower) -> Box<Type> {
        let mut types = vec![*left];

        loop {
            self.expect(TokenKind::POLE);
            types.push(*self.parse_type(bp));

            if self.current_token().kind != TokenKind::POLE {
                break;
            }
        }

        Box::new(Type::Union(types))
    }
}
