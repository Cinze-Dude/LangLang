use crate::{
    frontend::ast::{
        BinaryOperator, Expr,
        Literal::{self},
        PostfixOperator, PrefixOperator, Program, Stmt,
    },
    runtime::{
        errors::{RuntimeError, RuntimeResult, RuntimeValueResult},
        values::{Interpreter, RuntimeType, RuntimeValue},
    },
};

fn fact(num: i32) -> Result<i32, ()> {
    match num {
        0 => Ok(1),
        1.. => Ok(fact(num - 1).unwrap() * num),
        _ => Err(()),
    }
}

fn eval_bin_add(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Null, i) => Ok(i),
        (i, RuntimeValue::Null) => Ok(i),

        (RuntimeValue::NativeFunction(_), _) => Err(RuntimeError::InvalidOperand),
        (RuntimeValue::Func(_), _) => Err(RuntimeError::InvalidOperand),
        (RuntimeValue::Map(_), _) => Err(RuntimeError::InvalidOperand),

        (_, RuntimeValue::Map(_)) => Err(RuntimeError::InvalidOperand),
        (_, RuntimeValue::Func(_)) => Err(RuntimeError::InvalidOperand),
        (_, RuntimeValue::NativeFunction(_)) => Err(RuntimeError::InvalidOperand),

        (RuntimeValue::Infinity, RuntimeValue::NegInfinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::Infinity) => Ok(RuntimeValue::NaN),

        (RuntimeValue::Infinity, _) | (_, RuntimeValue::Infinity) => Ok(RuntimeValue::Infinity),
        (RuntimeValue::NegInfinity, _) | (_, RuntimeValue::NegInfinity) => {
            Ok(RuntimeValue::NegInfinity)
        }

        (RuntimeValue::NaN, _) | (_, RuntimeValue::NaN) => Ok(RuntimeValue::NaN),

        (RuntimeValue::String(mut s), RuntimeValue::String(z)) => {
            s.push_str(&z);
            Ok(RuntimeValue::String(s))
        }
        (RuntimeValue::String(s), RuntimeValue::Rune(c)) => {
            Ok(RuntimeValue::String(format!("{}{}", s, c)))
        }
        (RuntimeValue::Rune(s), RuntimeValue::String(c)) => {
            Ok(RuntimeValue::String(format!("{}{}", s, c)))
        }

        (i, RuntimeValue::Vector(mut v, ty)) => {
            if i.runtime_type() == ty {
                v.push(i);
                Ok(RuntimeValue::Vector(v, ty))
            } else {
                Err(RuntimeError::InvalidOperand)
            }
        }

        (RuntimeValue::Rune(r), RuntimeValue::Rune(h)) => {
            Ok(RuntimeValue::String(format!("{}{}", r, h)))
        }

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(RuntimeValue::Number(n + m)),

        _ => Err(RuntimeError::InvalidOperand),
    }
}

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

        let ty = values
            .first()
            .map(RuntimeValue::runtime_type)
            .unwrap_or(RuntimeType::Null);

        for elem in &values {
            if elem.runtime_type() != ty {
                return Err(RuntimeError::TypeMismatch {
                    expected: ty.stringify(),
                    found: elem.runtime_type().stringify(),
                });
            }
        }

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

                if n < 0.0 || n.fract() != 0.0 {
                    return Err(RuntimeError::Custom(
                        "Factorial expects a non-negative integer.".into(),
                    ));
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
        let l = self.eval_expr(left)?;
        let r = self.eval_expr(right)?;
        match (op, l, r) {
            (BinaryOperator::PLUS, x, y) => eval_bin_add(x, y),
            (BinaryOperator::MINUS, x, y) => Ok(RuntimeValue::Number(0.0)),
            (BinaryOperator::TIMES, x, y) => Ok(RuntimeValue::Number(0.0)),
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
            _ => Err(RuntimeError::NotImplemented),
        }
    }
}
