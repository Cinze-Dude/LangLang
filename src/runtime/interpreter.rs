use std::{cell::RefCell, rc::Rc};

use crate::{
    frontend::ast::{
        AssignOperator, BinaryOperator, Expr, Literal, PostfixOperator, PrefixOperator, Program,
        Stmt, Type,
    },
    runtime::{
        binops::{
            eval_bin_add, eval_bin_and, eval_bin_cmp, eval_bin_div, eval_bin_eq, eval_bin_ls,
            eval_bin_mod, eval_bin_mul, eval_bin_neq, eval_bin_or, eval_bin_pow, eval_bin_rs,
            eval_bin_sub, eval_bin_xor, fact, is_truthy,
        },
        errors::{RuntimeError, RuntimeResult, RuntimeTypeResult, RuntimeValueResult},
        values::{Environment, Interpreter, RuntimeType, RuntimeValue, Variable},
    },
};

fn eval_block_value(int: &mut Interpreter, value: RuntimeValue) -> RuntimeValueResult {
    match value {
        RuntimeValue::Block(body, env) => int.eval_block(&body, env),
        _ => unreachable!(),
    }
}

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

    fn eval_map(&mut self, raw_map: &[(Expr, Option<Expr>)]) -> RuntimeValueResult {
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

    fn eval_tuple(&mut self, values: &[Expr]) -> RuntimeValueResult {
        let values = values
            .iter()
            .map(|expr| self.eval_expr(expr))
            .collect::<RuntimeResult<Vec<_>>>()?;

        let len = values.len();

        Ok(RuntimeValue::Tuple(values, len))
    }

    fn eval_unary(&mut self, op: &PrefixOperator, value: &Expr) -> RuntimeValueResult {
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
            (op, value) => Err(RuntimeError::Custom(format!(
                "Unary operator {:?} cannot be applied to {:?}",
                op, value
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
            _ => {
                let e = Box::new(expr.clone());
                match op {
                    PostfixOperator::DEC => self.eval_assign(
                        &e,
                        &AssignOperator::ME,
                        &Box::new(Expr::Literal(Literal::Number(1.0))),
                    ),
                    PostfixOperator::INC => self.eval_assign(
                        &e,
                        &AssignOperator::PE,
                        &Box::new(Expr::Literal(Literal::Number(1.0))),
                    ),
                    PostfixOperator::OPP => self.eval_assign(
                        &e,
                        &AssignOperator::ME,
                        &Box::new(Expr::Literal(Literal::Number(-1.0))),
                    ),
                    PostfixOperator::RZA => self.eval_assign(
                        &e,
                        &AssignOperator::POWE,
                        &Box::new(Expr::Literal(Literal::Number(0.5))),
                    ),
                    PostfixOperator::FIB => self.eval_assign(
                        &e.clone(),
                        &AssignOperator::ASSIGN,
                        &Box::new(Expr::Postfix(PostfixOperator::FACT, e)),
                    ),
                    _ => unreachable!(),
                }
            }
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

    fn eval_convert(&mut self, subj: &Box<Expr>, ty: &Type) -> RuntimeValueResult {
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

    fn eval_block(
        &mut self,
        body: &Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> RuntimeValueResult {
        let previous = self.env.clone();

        // enter block scope
        self.env = Rc::new(RefCell::new(Environment::with_parent(closure)));

        let mut result = RuntimeValue::Null;

        for stmt in body {
            result = self.eval_stmt(stmt)?;
        }

        // leave block scope
        self.env = previous;

        Ok(result)
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
            Expr::Block(stats) => {
                let b = RuntimeValue::Block(stats.clone(), self.env.clone());
                eval_block_value(self, b)
            }
            Expr::Map(kvs) => self.eval_map(kvs),
            Expr::Assign(name, op, value) => self.eval_assign(name, op, value),
            Expr::TypeOf(e) => Ok(RuntimeValue::Type(self.eval_expr(e)?.runtime_type())),
            Expr::Type(t) => Ok(RuntimeValue::Type(self.eval_type(t)?)),
            _ => Err(RuntimeError::NotImplemented),
        }
    }
}
