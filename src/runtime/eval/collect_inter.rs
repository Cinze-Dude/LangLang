use crate::{
    frontend::ast::Expr,
    runtime::{
        errors::{RuntimeError, RuntimeResult, RuntimeValueResult},
        values::{Interpreter, RuntimeType, RuntimeValue},
    },
};

impl Interpreter {
    pub fn eval_vec(&mut self, values: &[Expr]) -> RuntimeValueResult {
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

    pub fn eval_map(&mut self, raw_map: &[(Expr, Option<Expr>)]) -> RuntimeValueResult {
        let mut map = Vec::new();

        let mut key_type: Option<RuntimeType> = None;
        let mut value_type: Option<RuntimeType> = None;

        for (k, v) in raw_map {
            let key = self.eval_expr(k)?;
            let value = v.as_ref().map(|e| self.eval_expr(e)).transpose()?;

            if let Some(expected) = &key_type {
                if !expected.contains(&key.runtime_type()) {
                    return Err(RuntimeError::TypeMismatch {
                        expected: expected.stringify(),
                        found: key.runtime_type().stringify(),
                    });
                }
            } else {
                key_type = Some(key.runtime_type());
            }

            if let Some(val) = &value {
                if let Some(expected) = &value_type {
                    if !expected.contains(&val.runtime_type()) {
                        return Err(RuntimeError::TypeMismatch {
                            expected: expected.stringify(),
                            found: val.runtime_type().stringify(),
                        });
                    }
                } else {
                    value_type = Some(val.runtime_type());
                }
            }

            map.push((key, value));
        }

        Ok(RuntimeValue::Map(map))
    }

    pub fn eval_tuple(&mut self, values: &[Expr]) -> RuntimeValueResult {
        let values = values
            .iter()
            .map(|expr| self.eval_expr(expr))
            .collect::<RuntimeResult<Vec<_>>>()?;

        let len = values.len();

        Ok(RuntimeValue::Tuple(values, len))
    }
}
