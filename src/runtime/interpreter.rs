use crate::{
    frontend::ast::{
        BinaryOperator, Expr, Literal, PostfixOperator, PrefixOperator, Program, Stmt,
    },
    runtime::{
        binops::{
            eval_bin_add, eval_bin_and, eval_bin_cmp, eval_bin_div, eval_bin_eq, eval_bin_ls,
            eval_bin_mod, eval_bin_mul, eval_bin_neq, eval_bin_or, eval_bin_pow, eval_bin_rs,
            eval_bin_sub, eval_bin_xor, fact,
        },
        errors::{RuntimeError, RuntimeResult, RuntimeValueResult},
        values::{Interpreter, RuntimeType, RuntimeValue},
    },
};

impl Interpreter {
    pub fn eval_program(&mut self, program: Program) -> RuntimeValueResult {
        match program {
            Program(s) => self.eval_stmt(s[0].clone()),
        }
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> RuntimeValueResult {
        match stmt {
            Stmt::Expr(e) => self.eval_expr(&e),
            _ => Err(RuntimeError::NotImplemented),
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

    fn eval_vec(&mut self, values: &[Expr]) -> RuntimeValueResult {
        let values = values
            .iter()
            .map(|expr| self.eval_expr(expr))
            .collect::<RuntimeResult<Vec<_>>>()?;

        let mut types = Vec::<RuntimeType>::new();

        for value in &values {
            if !types.contains(&value.runtime_type()) {
                types.push(value.runtime_type());
            }
        }

        let ty = match types.len() {
            0 => RuntimeType::Null,
            1 => types.pop().unwrap(),
            _ => RuntimeType::Union(types),
        };

        Ok(RuntimeValue::Vector(values, ty))
    }

    fn eval_tuple(&mut self, values: &[Expr]) -> RuntimeValueResult {
        let values = values
            .iter()
            .map(|expr| self.eval_expr(expr))
            .collect::<RuntimeResult<Vec<_>>>()?;

        let len = values.len();

        Ok(RuntimeValue::Tuple(values, len))
    }

    fn eval_unary(&mut self, op: &PrefixOperator, value: &Expr) -> RuntimeValueResult {
        let val = self.eval_expr(value)?;
        match val {
            RuntimeValue::Number(x) => Ok(RuntimeValue::Number(match op {
                PrefixOperator::SIN => x.sin(),
                PrefixOperator::COS => x.cos(),
                PrefixOperator::TAN => x.tan(),
                PrefixOperator::ASIN => x.asin(),
                PrefixOperator::ACOS => x.acos(),
                PrefixOperator::ATAN => x.atan(),
                PrefixOperator::MINUS => -x,
                PrefixOperator::SQRT => x.sqrt(),
                _ => x,
            })),
            RuntimeValue::Infinity => Ok(match op {
                PrefixOperator::ATAN => RuntimeValue::Number(1.57079632679),
                PrefixOperator::MINUS => RuntimeValue::NegInfinity,
                PrefixOperator::SQRT => RuntimeValue::Infinity,
                _ => RuntimeValue::Number(0.0),
            }),
            RuntimeValue::NegInfinity => match op {
                PrefixOperator::ASIN | PrefixOperator::ACOS => Ok(RuntimeValue::NaN),
                PrefixOperator::ATAN => Ok(RuntimeValue::Number(-1.57079632679)),
                PrefixOperator::MINUS => Ok(RuntimeValue::Infinity),
                PrefixOperator::SQRT => Err(RuntimeError::InfinityError),
                _ => Ok(RuntimeValue::Number(0.0)),
            },
            RuntimeValue::NaN => Ok(RuntimeValue::NaN),
            RuntimeValue::Bool(x) => Ok(RuntimeValue::Bool(!x)),
            e => Err(RuntimeError::Custom(format!(
                "Unary Expressions cannot be {:?}",
                e
            ))),
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
            _ => Err(RuntimeError::NumberError),
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

    fn eval_expr(&mut self, expr: &Expr) -> RuntimeValueResult {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Vector(v) => self.eval_vec(v),
            Expr::Tuple(t) => self.eval_tuple(t),

            Expr::Prefix(op, val) => self.eval_unary(op, val),
            Expr::Postfix(op, val) => self.eval_postfix(op, val),
            Expr::Binary(left, op, right) => self.eval_binary(op, left, right),
            Expr::Range(start, end, step, ae) => self.eval_range(start, end, step, *ae),
            _ => Err(RuntimeError::NotImplemented),
        }
    }
}
