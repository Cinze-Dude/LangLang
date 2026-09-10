use std::{cell::RefCell, rc::Rc};

use crate::{
    frontend::ast::{
        AssignOperator, BinaryOperator, Expr, Literal, PostfixOperator, PrefixOperator, Program,
        Stmt, Type,
    },
    runtime::{
        binops::{
            eval_bin_add, eval_bin_and, eval_bin_cmp, eval_bin_div, eval_bin_eq, eval_bin_ls,
            eval_bin_mod, eval_bin_mul, eval_bin_neq, eval_bin_or, eval_bin_pow, eval_bin_rs,
            eval_bin_sub, eval_bin_xor, fact, is_truthy,
        },
        errors::{RuntimeError, RuntimeTypeResult, RuntimeValueResult},
        values::{Environment, Interpreter, RuntimeType, RuntimeValue},
    },
};

impl Interpreter {
    pub fn eval_program(&mut self, program: &Program) -> RuntimeValueResult {
        match program {
            Program(stmts) => {
                let mut result = RuntimeValue::Null;

                for stmt in stmts {
                    result = self.eval_stmt(stmt)?;
                }

                Ok(result)
            }
        }
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> RuntimeValueResult {
        match stmt {
            Stmt::Expr(e) => self.eval_expr(e),
            Stmt::While(cond, body) => {
                let previous = self.env.clone();

                self.env = Rc::new(RefCell::new(Environment::with_parent(previous.clone())));

                let mut res = RuntimeValue::Null;

                while is_truthy(self.eval_expr(cond)?) {
                    res = self.eval_expr(body)?;
                }

                self.env = previous;

                Ok(res)
            }
            Stmt::If {
                condition,
                then_branch,
                elifs,
                else_branch,
            } => {
                if is_truthy(self.eval_expr(condition)?) {
                    return self.eval_expr(then_branch);
                }

                for (cond, branch) in elifs {
                    if is_truthy(self.eval_expr(cond)?) {
                        return self.eval_expr(branch);
                    }
                }

                match else_branch {
                    Some(expr) => self.eval_expr(expr),
                    None => Ok(RuntimeValue::Null),
                }
            }
            Stmt::Var {
                name,
                expr,
                imut,
                dynm,
                typ,
            } => self.eval_variable(name, expr, imut, dynm, typ),
            Stmt::For(repeatable, body) => {
                let rep = self.eval_expr(repeatable)?;

                let RuntimeValue::Repeatable(variable, col) = rep else {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "Repeatable".to_string(),
                        found: rep.runtime_type().stringify(),
                    });
                };

                if self.env.borrow().get(&variable).is_some() {
                    return Err(RuntimeError::VariableNameAlreadyExists(variable));
                }

                let mut result = RuntimeValue::Null;

                for value in col {
                    self.env.borrow_mut().assign(&variable, value);

                    if let Expr::Block(inner) = body.as_ref() {
                        result = self.eval_block(inner, self.env.clone())?;
                    }
                }

                Ok(result)
            }
            Stmt::Alias(_, _) => Ok(RuntimeValue::Null),
            _ => Err(RuntimeError::NotImplemented),
        }
    }

    pub fn eval_type(&self, ty: &Type) -> RuntimeTypeResult {
        match ty {
            Type::Block => Ok(RuntimeType::Block),
            Type::Bool => Ok(RuntimeType::Bool),
            Type::Inferred => Err(RuntimeError::InvalidType),
            Type::Map(k, v) => Ok(RuntimeType::Map(
                Box::new(self.eval_type(k)?),
                Box::new(self.eval_type(v)?),
            )),
            Type::Type => Ok(RuntimeType::Type),
            Type::Null => Ok(RuntimeType::Null),
            Type::Number => Ok(RuntimeType::Number),
            Type::Rune => Ok(RuntimeType::Rune),
            Type::String => Ok(RuntimeType::String),
            Type::Symbol(_) => Err(RuntimeError::InvalidType),
            Type::Repeatable => Ok(RuntimeType::Repeatable),
            Type::Tuple(v, _) => Ok(RuntimeType::Tuple(
                v.iter()
                    .map(|t| self.eval_type(t))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Type::Union(v) => Ok(RuntimeType::Tuple(
                v.iter()
                    .map(|t| self.eval_type(t))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Type::Vector(t) => Ok(RuntimeType::Vector(Box::new(self.eval_type(t)?))),
        }
    }

    fn eval_literal(&self, lit: &Literal) -> RuntimeValueResult {
        match lit {
            Literal::Number(number) => Ok(RuntimeValue::Number(*number)),
            Literal::String(string) => Ok(RuntimeValue::String(string.clone())),
            Literal::Null => Ok(RuntimeValue::Null),
            Literal::Rune(rune) => Ok(RuntimeValue::Rune(*rune)),
            Literal::Symbol(symbol) => self.get_symbol(symbol.as_str()),
            Literal::Bool(boolean) => Ok(RuntimeValue::Bool(*boolean)),
            Literal::Infinity => Ok(RuntimeValue::Infinity),
            Literal::NaN => Ok(RuntimeValue::NaN),
        }
    }

    fn eval_unary(&mut self, op: &PrefixOperator, value: &Expr) -> RuntimeValueResult {
        use std::f64::consts::FRAC_PI_2;

        let value = self.eval_expr(value)?;

        match (op, value) {
            // Logical
            (PrefixOperator::NOT, RuntimeValue::Bool(x)) => Ok(RuntimeValue::Bool(!x)),
            (PrefixOperator::NOT, RuntimeValue::Null)
            | (PrefixOperator::NOT, RuntimeValue::NaN) => Ok(RuntimeValue::Bool(true)),
            (PrefixOperator::NOT, _) => Ok(RuntimeValue::Bool(false)),

            // Number
            (PrefixOperator::SIN, RuntimeValue::Number(x)) => Ok(RuntimeValue::Number(x.sin())),
            (PrefixOperator::COS, RuntimeValue::Number(x)) => Ok(RuntimeValue::Number(x.cos())),
            (PrefixOperator::TAN, RuntimeValue::Number(x)) => Ok(RuntimeValue::Number(x.tan())),
            (PrefixOperator::ASIN, RuntimeValue::Number(x)) => Ok(RuntimeValue::Number(x.asin())),
            (PrefixOperator::ACOS, RuntimeValue::Number(x)) => Ok(RuntimeValue::Number(x.acos())),
            (PrefixOperator::ATAN, RuntimeValue::Number(x)) => Ok(RuntimeValue::Number(x.atan())),
            (PrefixOperator::SQRT, RuntimeValue::Number(x)) => Ok(RuntimeValue::Number(x.sqrt())),
            (PrefixOperator::MINUS, RuntimeValue::Number(x)) => Ok(RuntimeValue::Number(-x)),

            // +∞
            (PrefixOperator::ATAN, RuntimeValue::Infinity) => Ok(RuntimeValue::Number(FRAC_PI_2)),
            (PrefixOperator::MINUS, RuntimeValue::Infinity) => Ok(RuntimeValue::NegInfinity),
            (PrefixOperator::SQRT, RuntimeValue::Infinity) => Ok(RuntimeValue::Infinity),
            (_, RuntimeValue::Infinity) => Ok(RuntimeValue::NaN),

            // -∞
            (PrefixOperator::ATAN, RuntimeValue::NegInfinity) => {
                Ok(RuntimeValue::Number(-FRAC_PI_2))
            }
            (PrefixOperator::MINUS, RuntimeValue::NegInfinity) => Ok(RuntimeValue::Infinity),
            (PrefixOperator::SQRT, RuntimeValue::NegInfinity) => Err(RuntimeError::InfinityError),
            (_, RuntimeValue::NegInfinity) => Ok(RuntimeValue::NaN),

            // NaN propagates
            (_, RuntimeValue::NaN) => Ok(RuntimeValue::NaN),

            // Everything else is invalid
            _ => Err(RuntimeError::InvalidOperand),
        }
    }

    fn eval_postfix(&mut self, op: &PostfixOperator, expr: &Expr) -> RuntimeValueResult {
        let value = self.eval_expr(expr)?;

        match op {
            PostfixOperator::FACT => {
                let n = value.as_number()?;

                if n < 0.0 {
                    return Err(RuntimeError::FactorialNeg);
                }

                if n.fract() != 0.0 {
                    return Err(RuntimeError::UnexpectedFract);
                }

                Ok(RuntimeValue::Number(fact(n as i32).unwrap() as f64))
            }
            _ => {
                let name = self.get_assign_name(expr)?;
                let old = self
                    .env
                    .borrow()
                    .get(name.as_ref())
                    .ok_or(RuntimeError::UndefinedVariable(name.clone()))?;
                let (assign_op, new) = match op {
                    PostfixOperator::DEC => (AssignOperator::ME, RuntimeValue::Number(1.0)),
                    PostfixOperator::INC => (AssignOperator::PE, RuntimeValue::Number(1.0)),
                    PostfixOperator::FIB => (
                        AssignOperator::ASSIGN,
                        RuntimeValue::Number(
                            fact(old.as_number().map_err(|_| RuntimeError::TypeMismatch {
                                expected: RuntimeType::Number.stringify(),
                                found: old.runtime_type().stringify(),
                            })? as i32)?
                            .into(),
                        )
                        .into(),
                    ),
                    PostfixOperator::OPP => (AssignOperator::TE, RuntimeValue::Number(-1.0)),
                    PostfixOperator::RZA => (AssignOperator::POWE, RuntimeValue::Number(0.5)),
                    _ => unreachable!(),
                };

                self.env
                    .borrow_mut()
                    .assign(&name, self.apply_assign_op(&assign_op, old, new)?);

                Ok(RuntimeValue::NaN)
            }
        }
    }

    fn eval_binary(
        &mut self,
        op: &BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> RuntimeValueResult {
        let x = self.eval_expr(left)?;
        let y = self.eval_expr(right)?;
        match op {
            BinaryOperator::PLUS => eval_bin_add(x, y),
            BinaryOperator::MINUS => eval_bin_sub(x, y),
            BinaryOperator::TIMES => eval_bin_mul(x, y),
            BinaryOperator::SLASH => eval_bin_div(x, y),
            BinaryOperator::MOD => eval_bin_mod(x, y),
            BinaryOperator::POW => eval_bin_pow(x, y),
            BinaryOperator::AND => eval_bin_and(x, y),
            BinaryOperator::OR => eval_bin_or(x, y),
            BinaryOperator::XOR => eval_bin_xor(x, y),
            BinaryOperator::EQUALS => eval_bin_eq(x, y),
            BinaryOperator::NEQUAL => eval_bin_neq(x, y),
            BinaryOperator::LS => eval_bin_ls(x, y),
            BinaryOperator::RS => eval_bin_rs(x, y),
            BinaryOperator::GT => eval_bin_cmp(x, y, "gt"),
            BinaryOperator::GE => eval_bin_cmp(x, y, "ge"),
            BinaryOperator::LT => eval_bin_cmp(x, y, "lt"),
            BinaryOperator::LE => eval_bin_cmp(x, y, "le"),
            BinaryOperator::TEQUAL => Err(RuntimeError::NotImplemented),
        }
    }

    fn eval_range(
        &mut self,
        st: &Box<Expr>,
        ed: &Box<Expr>,
        sp: &Box<Expr>,
        ae: bool,
    ) -> RuntimeValueResult {
        let beg = self.eval_expr(st)?;
        let end = self.eval_expr(ed)?;
        let stp = self.eval_expr(sp)?;

        match (beg, end, stp) {
            (RuntimeValue::Number(b), RuntimeValue::Number(e), RuntimeValue::Number(s)) => {
                if b.fract() != 0.0 || e.fract() != 0.0 || e.fract() != 0.0 {
                    return Err(RuntimeError::UnexpectedFract);
                }

                let mut elems = Vec::new();
                for i in b as i32..e as i32 {
                    if i % s as i32 == 0 {
                        elems.push(RuntimeValue::Number(i as f64));
                    }
                }

                if ae {
                    elems.push(RuntimeValue::Number(e));
                }

                let l = elems.len();
                Ok(RuntimeValue::Tuple(elems, l))
            }
            _ => Err(RuntimeError::InvalidOperand),
        }
    }

    fn eval_convert(&mut self, subj: &Box<Expr>, ty: &Type) -> RuntimeValueResult {
        let s = self.eval_expr(subj)?;
        let t = self.eval_type(ty)?;

        match (s, t) {
            (_, RuntimeType::Null) => Ok(RuntimeValue::Null),

            (RuntimeValue::Rune(r), RuntimeType::String) => Ok(RuntimeValue::String(r.to_string())),

            (RuntimeValue::String(s), RuntimeType::Rune) => {
                let mut chars = s.chars();

                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(RuntimeValue::Rune(c)),
                    _ => Err(RuntimeError::InvalidOperand),
                }
            }

            (RuntimeValue::Null, RuntimeType::Number) => Ok(RuntimeValue::Number(-1.0)),

            (RuntimeValue::Bool(b), RuntimeType::Number) => {
                Ok(RuntimeValue::Number(if b { 1.0 } else { 0.0 }))
            }

            (RuntimeValue::Number(n), RuntimeType::Bool) => Ok(RuntimeValue::Bool(n != 0.0)),

            (RuntimeValue::Rune(r), RuntimeType::Number) => {
                Ok(RuntimeValue::Number(r as u32 as f64))
            }

            (RuntimeValue::Number(n), RuntimeType::Rune) => {
                if n.fract() != 0.0 {
                    return Err(RuntimeError::InvalidOperand);
                }

                let c = char::from_u32(n as u32).ok_or(RuntimeError::InvalidOperand)?;

                Ok(RuntimeValue::Rune(c))
            }

            (RuntimeValue::Number(n), RuntimeType::String) => {
                Ok(RuntimeValue::String(n.to_string()))
            }

            (RuntimeValue::Bool(b), RuntimeType::String) => Ok(RuntimeValue::String(b.to_string())),

            (RuntimeValue::Rune(r), RuntimeType::Bool) => Ok(RuntimeValue::Bool(r != '\0')),

            (value, ty) if value.runtime_type() == ty => Ok(value),

            _ => Err(RuntimeError::InvalidOperand),
        }
    }

    fn eval_block(
        &mut self,
        body: &Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> RuntimeValueResult {
        let previous = self.env.clone();

        self.env = Rc::new(RefCell::new(Environment::with_parent(closure)));

        let mut result = RuntimeValue::Null;

        for stmt in body {
            result = self.eval_stmt(stmt)?;
        }

        self.env = previous;

        Ok(result)
    }

    fn eval_of(&mut self, x: &Box<Expr>, collection: &Box<Expr>) -> RuntimeValueResult {
        let name = match x.as_ref() {
            Expr::Literal(Literal::Symbol(s)) => s.clone(),
            ty => {
                return Err(RuntimeError::TypeMismatch {
                    expected: "Symbol".to_string(),
                    found: self.eval_expr(ty)?.runtime_type().stringify(),
                });
            }
        };

        let col = self.eval_expr(collection)?;
        let v = match col {
            RuntimeValue::Vector(v, _) => v,
            RuntimeValue::Tuple(v, _) => v,
            _ => {
                return Err(RuntimeError::TypeMismatch {
                    expected: "Vector".to_string(),
                    found: col.runtime_type().stringify(),
                });
            }
        };

        Ok(RuntimeValue::Repeatable(name, v))
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> RuntimeValueResult {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Vector(v) => self.eval_vec(v),
            Expr::Tuple(t) => self.eval_tuple(t),

            Expr::Prefix(op, val) => self.eval_unary(op, val),
            Expr::Postfix(op, val) => self.eval_postfix(op, val),
            Expr::Binary(left, op, right) => self.eval_binary(op, left, right),
            Expr::Range(start, end, step, ae) => self.eval_range(start, end, step, *ae),
            Expr::Convert(subj, ty) => self.eval_convert(subj, ty),
            Expr::Block(stats) => self.eval_block(stats, self.env.clone()),
            Expr::Map(kvs) => self.eval_map(kvs),
            Expr::Assign(name, op, value) => self.eval_assign(name, op, value),
            Expr::TypeOf(e) => Ok(RuntimeValue::Type(self.eval_expr(e)?.runtime_type())),
            Expr::Type(t) => Ok(RuntimeValue::Type(self.eval_type(t)?)),
            Expr::Of(x, collection) => self.eval_of(x, collection),
            _ => Err(RuntimeError::NotImplemented),
        }
    }
}
