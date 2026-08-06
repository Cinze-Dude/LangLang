use crate::frontend::{
    ast::{TYPELU, Type},
    errors::{FrontendError, ResultType},
    lookups::{BindingPower, TBP, TLED, TNUD},
    parser::Parser,
    tokens::TokenKind,
};

impl Parser {
    pub fn parse_type(&mut self, bp: BindingPower) -> ResultType {
        let t = self.current_token();

        let nud = {
            let table = TNUD.read().unwrap();

            *table
                .get(&t.kind)
                .ok_or_else(|| FrontendError::InvalidType(format!("{:?}", t.kind)))?
        };

        let mut left = nud(self)?;

        loop {
            if !self.has_tokens() {
                break;
            }

            let t = self.current_token();

            let Some(&tbp) = TBP.read().unwrap().get(&t.kind) else {
                break;
            };

            if tbp <= bp {
                break;
            }

            let handler = {
                let table = TLED.read().unwrap();

                *table
                    .get(&t.kind)
                    .ok_or_else(|| FrontendError::InvalidType(format!("{:?}", t.kind)))?
            };

            left = handler(self, left, tbp)?;
        }

        Ok(left)
    }

    pub fn parse_symbol(&mut self) -> ResultType {
        let token = self.eat();

        if let Some(ty) = TYPELU.get(token.value.as_str()) {
            Ok(Box::new(ty.clone()))
        } else {
            Ok(Box::new(Type::Symbol(token.value.clone())))
        }
    }

    pub fn parse_tuple_type(&mut self, first: Box<Type>) -> ResultType {
        let mut types = vec![*first];

        while self.current_token().kind == TokenKind::POLE {
            self.eat();
            types.push(*self.parse_type(BindingPower::DEFAULT)?);
        }

        self.expect(TokenKind::SC)?;

        let length = self
            .expect(TokenKind::NUMBER)?
            .value
            .parse()
            .map_err(|_| FrontendError::InvalidNumber(self.current_token().value.clone()))?;

        self.expect(TokenKind::CCURLY)?;

        Ok(Box::new(Type::Tuple(types, length)))
    }

    pub fn parse_map_type(&mut self, first: Box<Type>) -> ResultType {
        self.expect(TokenKind::COLON)?;

        let val = self.parse_type(BindingPower::DEFAULT)?;

        self.expect(TokenKind::CCURLY)?;

        Ok(Box::new(Type::Map(first, val)))
    }

    pub fn parse_brace_type(&mut self) -> ResultType {
        self.expect(TokenKind::SCURLY)?;

        let first = self.parse_type(BindingPower::DEFAULT)?;

        if self.current_token().kind == TokenKind::COLON {
            self.parse_map_type(first)
        } else {
            self.parse_tuple_type(first)
        }
    }

    pub fn parse_vector(&mut self) -> ResultType {
        self.expect(TokenKind::SBRACK)?;

        let ty = self.parse_type(BindingPower::DEFAULT)?;

        self.expect(TokenKind::CBRACK)?;

        Ok(Box::new(Type::Vector(ty)))
    }

    pub fn parse_option(&mut self, ty: Box<Type>, _: BindingPower) -> ResultType {
        self.expect(TokenKind::QUESTION)?;

        Ok(Box::new(Type::Union(vec![*ty, Type::Null])))
    }

    pub fn parse_union(&mut self, left: Box<Type>, bp: BindingPower) -> ResultType {
        let mut types = vec![*left];

        loop {
            self.expect(TokenKind::POLE)?;

            types.push(*self.parse_type(bp)?);

            if self.current_token().kind != TokenKind::POLE {
                break;
            }
        }

        Ok(Box::new(Type::Union(types)))
    }
}
