use crate::{
    frontend::ast::{AssignOperator, Expr, Literal, Type},
    runtime::{
        binops::{
            eval_bin_add, eval_bin_div, eval_bin_mod, eval_bin_mul, eval_bin_pow, eval_bin_sub,
        },
        errors::{RuntimeError, RuntimeValueResult},
        values::{Interpreter, RuntimeValue, Variable},
    },
};

impl Interpreter {
    pub fn get_assign_name(&self, expr: &Expr) -> Result<String, RuntimeError> {
        match expr {
            Expr::Literal(Literal::Symbol(name)) => Ok(name.clone()),
            _ => Err(RuntimeError::InvalidVariableTarget(format!("{:?}", expr))),
        }
    }

    pub fn eval_variable(
        &mut self,
        name: &String,
        expr: &Option<Expr>,
        imut: &bool,
        dynm: &bool,
        typ: &Type,
    ) -> RuntimeValueResult {
        let value = self.eval_expr(expr.as_ref().unwrap())?;
        let expected = if let Type::Inferred = typ {
            value.runtime_type()
        } else {
            self.eval_type(typ)?
        };

        if !expected.contains(&value.runtime_type()) {
            return Err(RuntimeError::TypeMismatch {
                expected: expected.stringify(),
                found: value.runtime_type().stringify(),
            });
        }
        let variable = if *dynm {
            self.env.borrow_mut().dynm_id += 1;
            Variable::DynamVariable {
                id: self.env.borrow_mut().dynm_id - 1,
                name: name.clone(),
                value: expr.clone().unwrap(),
                used_in: Vec::new(),
            }
        } else if *imut {
            Variable::ImutVariable {
                name: name.clone(),
                value: self.eval_expr(&expr.clone().unwrap())?,
            }
        } else {
            Variable::RigidVariable {
                name: name.clone(),
                value: match expr {
                    Some(Expr::Block(b)) => Some(RuntimeValue::Block(b.clone(), self.env.clone())),
                    Some(e) => Some(self.eval_expr(e)?),
                    None => None,
                },
            }
        };

        self.env.borrow_mut().variables.push(variable);

        Ok(RuntimeValue::Null)
    }
    pub fn apply_assign_op(
        &self,
        op: &AssignOperator,
        old: RuntimeValue,
        value: RuntimeValue,
    ) -> RuntimeValueResult {
        match op {
            AssignOperator::ME => eval_bin_sub(old, value),
            AssignOperator::PE => eval_bin_add(old, value),
            AssignOperator::MODE => eval_bin_mod(old, value),
            AssignOperator::POWE => eval_bin_pow(old, value),
            AssignOperator::SE => eval_bin_div(old, value),
            AssignOperator::TE => eval_bin_mul(old, value),
            AssignOperator::ASSIGN => Ok(value),
        }
    }
}
