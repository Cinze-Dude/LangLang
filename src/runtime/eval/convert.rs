use crate::{
    frontend::ast::{Expr, Type},
    runtime::{
        errors::{RuntimeError, RuntimeValueResult},
        values::{Interpreter, RuntimeType, RuntimeValue},
    },
};

impl Interpreter {
    pub fn eval_convert(&mut self, subj: &Box<Expr>, ty: &Type) -> RuntimeValueResult {
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
}
