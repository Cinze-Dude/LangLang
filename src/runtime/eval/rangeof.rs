use crate::{
    frontend::ast::{Expr, Literal},
    runtime::{
        errors::{RuntimeError, RuntimeValueResult},
        values::{Interpreter, RuntimeValue},
    },
};

impl Interpreter {
    pub fn eval_of(&mut self, x: &Box<Expr>, collection: &Box<Expr>) -> RuntimeValueResult {
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

    pub fn eval_range(
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
}
