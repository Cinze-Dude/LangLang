use crate::{
    frontend::ast::{AssignOperator, Expr, Literal, Type},
    runtime::{
        binops::{
            eval_bin_add, eval_bin_div, eval_bin_mod, eval_bin_mul, eval_bin_pow, eval_bin_sub,
        },
        errors::{RuntimeError, RuntimeValueResult},
        values::{Interpreter, RuntimeType, RuntimeValue, Variable},
    },
};

impl Interpreter {
    pub fn get_assign_name(&self, expr: &Expr) -> Result<String, RuntimeError> {
        match expr {
            Expr::Literal(Literal::Symbol(name)) => Ok(name.clone()),
            _ => Err(RuntimeError::InvalidVariableTarget(format!("{:?}", expr))),
        }
    }

    pub fn eval_assign(
        &mut self,
        target: &Box<Expr>,
        op: &AssignOperator,
        value_expr: &Box<Expr>,
    ) -> RuntimeValueResult {
        let name = self.get_assign_name(target)?;

        let value = match &**value_expr {
            Expr::Block(b) => RuntimeValue::Block(b.clone(), self.env.clone()),
            _ => self.eval_expr(value_expr)?,
        };

        let old = {
            let env = self.env.borrow();

            match env.variables.iter().find(|v| match v {
                Variable::RigidVariable { name: n, .. }
                | Variable::DynamVariable { name: n, .. }
                | Variable::ImutVariable { name: n, .. } => n == &name,
            }) {
                Some(Variable::RigidVariable { value, .. }) => value
                    .clone()
                    .ok_or(RuntimeError::OperationOnUndefinedValue)?,

                Some(Variable::ImutVariable { .. }) => {
                    return Err(RuntimeError::OperationImmutableValue(name));
                }

                Some(Variable::DynamVariable { .. }) => {
                    return Err(RuntimeError::OperationDynamicValue(name));
                }

                None => {
                    return Err(RuntimeError::UndefinedVariable(name));
                }
            }
        };

        if !old.runtime_type().contains(&value.runtime_type()) {
            return Err(RuntimeError::TypeMismatch {
                expected: old.runtime_type().stringify(),
                found: value.runtime_type().stringify(),
            });
        }

        let result = self.apply_assign_op(op, old, value)?;

        let mut env = self.env.borrow_mut();

        if let Some(Variable::RigidVariable { value, .. }) =
            env.variables.iter_mut().find(|v| match v {
                Variable::RigidVariable { name: n, .. }
                | Variable::DynamVariable { name: n, .. }
                | Variable::ImutVariable { name: n, .. } => n == &name,
            })
        {
            *value = Some(result.clone());
            Ok(result)
        } else {
            Err(RuntimeError::UndefinedVariable(name))
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
        let value = if let Some(Expr::Block(b)) = expr {
            RuntimeValue::Block(b.clone(), self.env.clone())
        } else {
            self.eval_expr(expr.as_ref().unwrap())?
        };
        let expected = if let Type::Inferred = typ {
            value.runtime_type()
        } else if let Type::Block = typ {
            RuntimeType::Block
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
