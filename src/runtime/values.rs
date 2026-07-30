use crate::{
    frontend::ast::Expr,
    runtime::errors::{RuntimeError, RuntimeResult, RuntimeValueResult},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub struct Environment {
    variables: HashMap<String, RuntimeValue>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    params: Vec<String>,
    body: Expr,
    closure: Environment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeType {
    Null,

    String,
    Number,
    Bool,
    Rune,

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

            RuntimeType::Block => "block".into(),

            RuntimeType::Vector(ty) => {
                format!("[{}]", ty.stringify())
            }

            RuntimeType::Tuple(types) => {
                let types = types.iter().map(RuntimeType::stringify).collect::<Vec<_>>();

                format!("{{{}}}", types.join(" | "))
            }

            RuntimeType::Map(k, v) => {
                format!("{{{}: {}}}", k.stringify(), v.stringify())
            }

            RuntimeType::Func => "func".into(),
            RuntimeType::NativeFunction => "nativefunc".into(),

            RuntimeType::Union(types) => {
                let types = types.iter().map(RuntimeType::stringify).collect::<Vec<_>>();

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

    Vector(Vec<RuntimeValue>, RuntimeType),
    Tuple(Vec<RuntimeValue>, usize),
    Map(HashMap<RuntimeValue, RuntimeValue>),

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

            (RuntimeValue::Map(a), RuntimeValue::Map(b)) => {
                a.len() == b.len()
                    && a.iter().all(|(key_a, val_a)| {
                        b.iter()
                            .any(|(key_b, val_b)| key_a.equals(key_b) && val_a.equals(val_b))
                    })
            }

            _ => false,
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            RuntimeValue::Null => "null".into(),

            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Number(n) => n.to_string(),
            RuntimeValue::Bool(b) => b.to_string(),
            RuntimeValue::Rune(c) => c.to_string(),
            RuntimeValue::Infinity => "infinity".into(),
            RuntimeValue::NegInfinity => "negative infinity".into(),
            RuntimeValue::NaN => "NaN".into(),

            RuntimeValue::Vector(values, _) => {
                let items = values
                    .iter()
                    .map(RuntimeValue::stringify)
                    .collect::<Vec<_>>();

                format!("[{}]", items.join(", "))
            }

            RuntimeValue::Tuple(values, _) => {
                let items = values
                    .iter()
                    .map(RuntimeValue::stringify)
                    .collect::<Vec<_>>();

                format!("{{{}}}", items.join(", "))
            }

            RuntimeValue::Map(map) => {
                let items = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.stringify(), v.stringify()))
                    .collect::<Vec<_>>();

                format!("{{{}}}", items.join(", "))
            }

            RuntimeValue::Func(_) => "<function>".into(),
            RuntimeValue::NativeFunction(_) => "<native function>".into(),
        }
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

            RuntimeValue::Vector(_, ty) => RuntimeType::Vector(Box::new(ty.clone())),

            RuntimeValue::Tuple(values, _) => {
                RuntimeType::Tuple(values.iter().map(RuntimeValue::runtime_type).collect())
            }

            RuntimeValue::Map(_) => todo!(),

            RuntimeValue::Func(_) => RuntimeType::Func,
            RuntimeValue::NativeFunction(_) => RuntimeType::NativeFunction,
        }
    }
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    pub fn get_symbol(&self, key: &str) -> RuntimeValueResult {
        match self.env.variables.get(key) {
            Some(value) => Ok(value.clone()),
            None => Err(RuntimeError::UndefinedVariable(key.to_string())),
        }
    }
}
