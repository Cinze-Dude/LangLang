use crate::{
    frontend::ast::{AssignOperator, BinaryOperator, Expr, PostfixOperator, PrefixOperator},
    runtime::{
        errors::{RuntimeError, RuntimeValueResult},
        eval::binops::*,
        values::{Interpreter, RuntimeType, RuntimeValue},
    },
};

impl Interpreter {
    pub fn eval_unary(&mut self, op: &PrefixOperator, value: &Expr) -> RuntimeValueResult {
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

    pub fn eval_postfix(&mut self, op: &PostfixOperator, expr: &Expr) -> RuntimeValueResult {
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

    pub fn eval_binary(
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
}
