use crate::{
    frontend::ast::{Expr, Stmt},
    runtime::errors::{RuntimeError, RuntimeResult, RuntimeValueResult},
};
use colored::{ColoredString, Colorize};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum Variable {
    RigidVariable {
        name: String,
        value: Option<RuntimeValue>,
    },
    DynamVariable {
        id: usize,
        name: String,
        value: Expr,
        used_in: Vec<usize>,
    },
    ImutVariable {
        name: String,
        value: RuntimeValue,
    },
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub variables: Vec<Variable>,
    pub dynm_id: usize,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            dynm_id: 0,
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            variables: Vec::new(),
            dynm_id: 0,
            parent: Some(parent),
        }
    }

    pub fn get(&self, key: &str) -> Option<RuntimeValue> {
        match self.variables.iter().find(|var| match var {
            Variable::RigidVariable { name, .. }
            | Variable::DynamVariable { name, .. }
            | Variable::ImutVariable { name, .. } => name == key,
        }) {
            Some(Variable::RigidVariable { value, .. }) => value.clone(),

            Some(Variable::ImutVariable { value, .. }) => Some(value.clone()),

            Some(Variable::DynamVariable { value, .. }) => {
                // evaluate the expression here or elsewhere
                None
            }

            None => match &self.parent {
                Some(parent) => parent.borrow().get(key),
                None => None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    params: Vec<String>,
    body: Expr,
    closure: Rc<RefCell<Environment>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeType {
    Null,

    String,
    Number,
    Bool,
    Rune,
    Type,

    Tuple(Vec<RuntimeType>),
    Vector(Box<RuntimeType>),

    Map(Box<RuntimeType>, Box<RuntimeType>),

    Func,
    NativeFunction,
    Block,

    Union(Vec<RuntimeType>),
}

impl RuntimeType {
    pub fn stringify(&self) -> String {
        match self {
            RuntimeType::Null => "null".into(),
            RuntimeType::String => "string".into(),
            RuntimeType::Number => "number".into(),
            RuntimeType::Bool => "bool".into(),
            RuntimeType::Rune => "rune".into(),
            RuntimeType::Type => "type".into(),

            RuntimeType::Block => "block".into(),

            RuntimeType::Vector(ty) => {
                format!("[{}]", ty.stringify())
            }

            RuntimeType::Tuple(types) => {
                let types = types.iter().map(|x| x.stringify()).collect::<Vec<_>>();

                format!("{{{}}}", types.join(" | "))
            }

            RuntimeType::Map(k, v) => {
                format!("{{{}: {}}}", k.stringify(), v.stringify())
            }

            RuntimeType::Func => "func".into(),
            RuntimeType::NativeFunction => "nativefunc".into(),

            RuntimeType::Union(types) => {
                let types = types.iter().map(|x| x.stringify()).collect::<Vec<_>>();

                format!("{{{}}}", types.join(" | "))
            }
        }
    }

    pub fn contains(&self, other: &RuntimeType) -> bool {
        match (self, other) {
            (RuntimeType::Union(atypes), RuntimeType::Union(btypes)) => {
                btypes.iter().all(|bty| atypes.contains(bty))
            }

            (RuntimeType::Union(atypes), other) => atypes.iter().any(|aty| aty.contains(other)),

            (a, RuntimeType::Union(btypes)) => btypes.iter().all(|bty| a.contains(bty)),

            (b, other) => b == other,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Null,

    String(String),
    Number(f64),
    Bool(bool),
    Rune(char),
    Infinity,
    NegInfinity,
    NaN,
    Type(RuntimeType),

    Vector(Vec<RuntimeValue>, RuntimeType),
    Tuple(Vec<RuntimeValue>, usize),
    Map(Vec<(RuntimeValue, Option<RuntimeValue>)>),

    Block(Vec<Stmt>, Rc<RefCell<Environment>>),

    Func(FunctionValue),
    NativeFunction(FunctionValue),
}

impl RuntimeValue {
    pub fn as_number(&self) -> RuntimeResult<f64> {
        match self {
            RuntimeValue::Number(n) => Ok(*n),
            _ => Err(RuntimeError::NumberError),
        }
    }

    pub fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeValue::Null, RuntimeValue::Null)
            | (RuntimeValue::Infinity, RuntimeValue::Infinity)
            | (RuntimeValue::NegInfinity, RuntimeValue::NegInfinity)
            | (RuntimeValue::NaN, RuntimeValue::NaN) => true,

            (RuntimeValue::Bool(b), RuntimeValue::Bool(d)) => b == d,
            (RuntimeValue::Number(n), RuntimeValue::Number(m)) => n == m,
            (RuntimeValue::String(s), RuntimeValue::String(z)) => s == z,
            (RuntimeValue::Rune(r), RuntimeValue::Rune(q)) => r == q,

            (RuntimeValue::Vector(v, t), RuntimeValue::Vector(w, p)) => {
                t == p && v.len() == w.len() && v.iter().zip(w.iter()).all(|(a, b)| a.equals(b))
            }

            (RuntimeValue::Tuple(v, len), RuntimeValue::Tuple(w, other_len)) => {
                len == other_len && v.iter().zip(w.iter()).all(|(a, b)| a.equals(b))
            }

            (RuntimeValue::Type(t), RuntimeValue::Type(u)) => t == u,

            (RuntimeValue::Map(a), RuntimeValue::Map(b)) => {
                a.len() == b.len()
                    && a.iter().all(|(key_a, val_a)| {
                        b.iter().any(|(key_b, val_b)| {
                            key_a.equals(key_b)
                                && if let (Some(val_a), Some(val_b)) = (val_a, val_b) {
                                    val_a.equals(val_b)
                                } else {
                                    val_a.is_none() && val_b.is_none()
                                }
                        })
                    })
            }

            _ => false,
        }
    }

    pub fn stringify(&self, color: bool) -> String {
        let mut res = match self {
            RuntimeValue::Null => "null".bright_black(),

            RuntimeValue::String(s) => s.clone().green(),
            RuntimeValue::Number(n) => n.to_string().blue(),
            RuntimeValue::Bool(b) => b.to_string().yellow(),
            RuntimeValue::Rune(c) => c.to_string().magenta(),
            RuntimeValue::Infinity => "infinity".blue().into(),
            RuntimeValue::NegInfinity => "negative infinity".blue().into(),
            RuntimeValue::NaN => "NaN".blue().into(),
            RuntimeValue::Type(t) => t.stringify().bright_yellow(),

            RuntimeValue::Block(_, _) => "<block>".bright_cyan().into(),

            RuntimeValue::Vector(values, _) => {
                let items = values
                    .iter()
                    .map(|x| x.stringify(color))
                    .collect::<Vec<_>>();

                ColoredString::from(format!("[{}]", items.join(", ")))
            }

            RuntimeValue::Tuple(values, _) => {
                let items = values
                    .iter()
                    .map(|x| x.stringify(color))
                    .collect::<Vec<_>>();

                ColoredString::from(format!("{{{}}}", items.join(", ")))
            }

            RuntimeValue::Map(map) => {
                let items = map
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}: {}",
                            if color {
                                k.stringify(color).bold()
                            } else {
                                ColoredString::from(k.stringify(color))
                            },
                            if let Some(val) = v {
                                val.stringify(color)
                            } else {
                                "@".into()
                            }
                        )
                    })
                    .collect::<Vec<_>>();

                ColoredString::from(format!("{{{}}}", items.join(", ")))
            }

            RuntimeValue::Func(_) => "<function>".purple().into(),
            RuntimeValue::NativeFunction(_) => "<native function>".bright_purple().into(),
        }
        .to_string();
        if color { res } else { res.white().to_string() }
    }

    pub fn runtime_type(&self) -> RuntimeType {
        match self {
            RuntimeValue::Null => RuntimeType::Null,
            RuntimeValue::String(_) => RuntimeType::String,
            RuntimeValue::Number(_) => RuntimeType::Number,
            RuntimeValue::Bool(_) => RuntimeType::Bool,
            RuntimeValue::Rune(_) => RuntimeType::Rune,
            RuntimeValue::Infinity => RuntimeType::Number,
            RuntimeValue::NegInfinity => RuntimeType::Number,
            RuntimeValue::NaN => RuntimeType::Number,
            RuntimeValue::Type(_) => RuntimeType::Type,

            RuntimeValue::Block(_, _) => RuntimeType::Block,

            RuntimeValue::Vector(_, ty) => RuntimeType::Vector(Box::new(ty.clone())),

            RuntimeValue::Tuple(values, _) => {
                let mut types = Vec::new();
                values.iter().for_each(|e| {
                    if !types.contains(&e.runtime_type()) {
                        types.push(e.runtime_type());
                    }
                });
                RuntimeType::Tuple(types)
            }

            RuntimeValue::Map(k) => RuntimeType::Map(
                Box::new(k[0].0.runtime_type()),
                Box::new(k[0].1.clone().unwrap().runtime_type()),
            ),

            RuntimeValue::Func(_) => RuntimeType::Func,
            RuntimeValue::NativeFunction(_) => RuntimeType::NativeFunction,
        }
    }
}

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn get_symbol(&self, key: &str) -> RuntimeValueResult {
        self.env
            .borrow()
            .get(key)
            .ok_or(RuntimeError::UndefinedVariable(key.to_string()))
    }
}
