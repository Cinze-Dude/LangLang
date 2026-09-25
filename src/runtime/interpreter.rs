use std::{cell::RefCell, rc::Rc};

use crate::{
    frontend::ast::{Expr, Literal, Program, Stmt, Type},
    runtime::{
        errors::{RuntimeError, RuntimeTypeResult, RuntimeValueResult},
        eval::binops::is_truthy,
        values::{Environment, FunctionValue, Interpreter, RuntimeType, RuntimeValue},
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
            Stmt::Function(name, args, ret, body) => {
                if self.env.borrow_mut().assign(
                    name,
                    RuntimeValue::Func(FunctionValue {
                        params: args
                            .iter()
                            .map(|(x, t)| Ok((x.to_string(), self.eval_type(t)?)))
                            .collect::<Result<Vec<_>, _>>()?,
                        body: *body.clone(),
                        closure: self.env.clone(),
                        rettype: self.eval_type(ret)?,
                    }),
                ) {
                    Ok(RuntimeValue::Null)
                } else {
                    Err(RuntimeError::VariableNameAlreadyExists(name.to_string()))
                }
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

    // fn eval_call(&mut self, callee: &String, args: &[Expr]) -> RuntimeValueResult {
    //     let Some(RuntimeValue::Func(func)) = self.env.borrow().get(callee) else {
    //         return Err(RuntimeError::UndefinedFunctionName(callee.to_string()));
    //     };

    //     if func.params.len() != args.len() {
    //         return Err(RuntimeError::ArgumentLengthMismatch {
    //             expected: func.params.len().to_string(),
    //             found: args.len().to_string(),
    //         });
    //     }

    //     let Expr::Block(innerbody) = &func.body else {
    //         return Err(RuntimeError::Custom("".to_string()));
    //     };

    //     args.iter()
    //         .zip(func.params.iter())
    //         .map(|(arg, (name, typ))| {
    //             let value = self.eval_expr(arg)?;

    //             if value.runtime_type() != *typ {
    //                 return Err(RuntimeError::TypeMismatch {
    //                     expected: typ.stringify(),
    //                     found: value.runtime_type().stringify(),
    //                 });
    //             }

    //             self.env.borrow_mut().assign(name, value);

    //             Ok(())
    //         })
    //         .collect::<Result<Vec<_>, _>>()?;
    // }

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
            // Expr::Call(callee, args) => self.eval_call(callee, args),
            _ => Err(RuntimeError::NotImplemented),
        }
    }
}
