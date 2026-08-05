use crate::{
    frontend::ast::{AssignOperator, Expr, Literal},
    runtime::{
        errors::{RuntimeError, RuntimeValueResult},
        values::{Interpreter, RuntimeValue},
    },
};

impl Interpreter {
    pub fn get_assign_name(&self, expr: &Expr) -> Result<String, RuntimeError> {
        match expr {
            Expr::Literal(Literal::Symbol(name)) => Ok(name.clone()),
            _ => Err(RuntimeError::InvalidVariableTarget(format!("{:?}", expr))),
        }
    }

    pub fn apply_assign_op(
        &self,
        op: &AssignOperator,
        old: RuntimeValue,
        value: RuntimeValue,
    ) -> RuntimeValueResult {
        if !old.runtime_type().contains(&value.runtime_type()) {
            return Err(RuntimeError::TypeMismatch {
                expected: format!("{:?}", old.runtime_type()),
                found: format!("{:?}", value.runtime_type()),
            });
        }

        if let RuntimeValue::Number(a) = old
            && let RuntimeValue::Number(b) = value
        {
            Ok(RuntimeValue::Number(match op {
                AssignOperator::TE => a * b,
                AssignOperator::PE => a + b,
                AssignOperator::SE => a / b,
                AssignOperator::ME => a - b,
                AssignOperator::POWE => a.powf(b),
                AssignOperator::MODE => a % b,
                _ => unreachable!(),
            }))
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    }
}
