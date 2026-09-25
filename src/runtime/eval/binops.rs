use crate::runtime::{
    errors::{RuntimeError, RuntimeValueResult},
    values::RuntimeValue,
};

pub fn fact(num: i32) -> Result<i32, RuntimeError> {
    match num {
        0 => Ok(1),
        13.. => Err(RuntimeError::FactorialOverflow),
        1.. => Ok(fact(num - 1)? * num),
        _ => Err(RuntimeError::InvalidOperand),
    }
}

fn number_result(n: f64) -> RuntimeValue {
    if n.is_nan() {
        RuntimeValue::NaN
    } else if n.is_infinite() {
        if n.is_sign_positive() {
            RuntimeValue::Infinity
        } else {
            RuntimeValue::NegInfinity
        }
    } else {
        RuntimeValue::Number(n)
    }
}

pub fn eval_bin_add(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Null, i) => Ok(i),
        (i, RuntimeValue::Null) => Ok(i),

        (RuntimeValue::NativeFunction(_), _) => Err(RuntimeError::InvalidOperand),
        (RuntimeValue::Func(_), _) => Err(RuntimeError::InvalidOperand),
        (RuntimeValue::Map(_), _) => Err(RuntimeError::InvalidOperand),

        (_, RuntimeValue::Map(_)) => Err(RuntimeError::InvalidOperand),
        (_, RuntimeValue::Func(_)) => Err(RuntimeError::InvalidOperand),
        (_, RuntimeValue::NativeFunction(_)) => Err(RuntimeError::InvalidOperand),

        (RuntimeValue::Infinity, RuntimeValue::NegInfinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::Infinity) => Ok(RuntimeValue::NaN),

        (RuntimeValue::Infinity, _) | (_, RuntimeValue::Infinity) => Ok(RuntimeValue::Infinity),
        (RuntimeValue::NegInfinity, _) | (_, RuntimeValue::NegInfinity) => {
            Ok(RuntimeValue::NegInfinity)
        }

        (RuntimeValue::NaN, _) | (_, RuntimeValue::NaN) => Ok(RuntimeValue::NaN),

        (RuntimeValue::String(mut s), RuntimeValue::String(z)) => {
            s.push_str(&z);
            Ok(RuntimeValue::String(s))
        }
        (RuntimeValue::String(s), RuntimeValue::Rune(c)) => {
            Ok(RuntimeValue::String(format!("{}{}", s, c)))
        }
        (RuntimeValue::Rune(s), RuntimeValue::String(c)) => {
            Ok(RuntimeValue::String(format!("{}{}", s, c)))
        }

        (RuntimeValue::Vector(mut v, ty), RuntimeValue::Vector(w, x)) => {
            if x == ty {
                v.extend(w);
                Ok(RuntimeValue::Vector(v, ty))
            } else {
                Err(RuntimeError::InvalidOperand)
            }
        }

        (i, RuntimeValue::Vector(mut v, ty)) => {
            if i.runtime_type() == ty {
                v.push(i);
                Ok(RuntimeValue::Vector(v, ty))
            } else {
                Err(RuntimeError::InvalidOperand)
            }
        }

        (RuntimeValue::Vector(mut v, ty), i) => {
            if ty.contains(&i.runtime_type()) {
                v.push(i);
                Ok(RuntimeValue::Vector(v, ty))
            } else {
                Err(RuntimeError::InvalidOperand)
            }
        }

        (RuntimeValue::Rune(r), RuntimeValue::Rune(h)) => {
            Ok(RuntimeValue::String(format!("{}{}", r, h)))
        }

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(number_result(n + m)),

        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn eval_bin_sub(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Infinity, RuntimeValue::Infinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::NegInfinity) => Ok(RuntimeValue::NaN),

        (RuntimeValue::Infinity, RuntimeValue::NegInfinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::Infinity) => Ok(RuntimeValue::Infinity),

        (RuntimeValue::Infinity, _) | (_, RuntimeValue::NegInfinity) => Ok(RuntimeValue::Infinity),

        (RuntimeValue::NegInfinity, _) | (_, RuntimeValue::Infinity) => {
            Ok(RuntimeValue::NegInfinity)
        }

        (RuntimeValue::NaN, _) | (_, RuntimeValue::NaN) => Ok(RuntimeValue::NaN),

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(number_result(n - m)),

        // Vector - value: remove first matching element
        (RuntimeValue::Vector(mut v, ty), value) => {
            if let Some(index) = v.iter().position(|item| item.equals(&value)) {
                v.remove(index);
            }

            Ok(RuntimeValue::Vector(v, ty))
        }

        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn eval_bin_mul(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Infinity, RuntimeValue::Number(0.0))
        | (RuntimeValue::Number(0.0), RuntimeValue::Infinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::Number(0.0))
        | (RuntimeValue::Number(0.0), RuntimeValue::NegInfinity) => Ok(RuntimeValue::NaN),

        (RuntimeValue::Infinity, RuntimeValue::Infinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::NegInfinity) => Ok(RuntimeValue::Infinity),

        (RuntimeValue::Infinity, RuntimeValue::NegInfinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::Infinity) => Ok(RuntimeValue::NegInfinity),

        (RuntimeValue::Infinity, RuntimeValue::Number(n))
        | (RuntimeValue::Number(n), RuntimeValue::Infinity) => {
            if n > 0.0 {
                Ok(RuntimeValue::Infinity)
            } else if n < 0.0 {
                Ok(RuntimeValue::NegInfinity)
            } else {
                Ok(RuntimeValue::NaN)
            }
        }

        (RuntimeValue::NegInfinity, RuntimeValue::Number(n))
        | (RuntimeValue::Number(n), RuntimeValue::NegInfinity) => {
            if n > 0.0 {
                Ok(RuntimeValue::NegInfinity)
            } else if n < 0.0 {
                Ok(RuntimeValue::Infinity)
            } else {
                Ok(RuntimeValue::NaN)
            }
        }

        (RuntimeValue::NaN, _) | (_, RuntimeValue::NaN) => Ok(RuntimeValue::NaN),

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(number_result(n * m)),

        // Vector scaling
        (RuntimeValue::Vector(v, ty), RuntimeValue::Number(scale)) => Ok(RuntimeValue::Vector(
            v.into_iter()
                .map(|value| eval_bin_mul(value, RuntimeValue::Number(scale)))
                .collect::<Result<Vec<_>, _>>()?,
            ty,
        )),

        (RuntimeValue::Number(scale), RuntimeValue::Vector(v, ty)) => Ok(RuntimeValue::Vector(
            v.into_iter()
                .map(|value| eval_bin_mul(value, RuntimeValue::Number(scale)))
                .collect::<Result<Vec<_>, _>>()?,
            ty,
        )),

        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn eval_bin_div(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Infinity, RuntimeValue::Infinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::NegInfinity)
        | (RuntimeValue::Infinity, RuntimeValue::NegInfinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::Infinity) => Ok(RuntimeValue::NaN),

        (RuntimeValue::Infinity, RuntimeValue::Number(0.0))
        | (RuntimeValue::NegInfinity, RuntimeValue::Number(0.0)) => {
            Err(RuntimeError::DivisionByZero)
        }

        (RuntimeValue::Infinity, RuntimeValue::Number(n)) => {
            if n > 0.0 {
                Ok(RuntimeValue::Infinity)
            } else {
                Ok(RuntimeValue::NegInfinity)
            }
        }

        (RuntimeValue::NegInfinity, RuntimeValue::Number(n)) => {
            if n > 0.0 {
                Ok(RuntimeValue::NegInfinity)
            } else {
                Ok(RuntimeValue::Infinity)
            }
        }

        (RuntimeValue::Number(_), RuntimeValue::Infinity)
        | (RuntimeValue::Number(_), RuntimeValue::NegInfinity) => Ok(RuntimeValue::Number(0.0)),

        (RuntimeValue::NaN, _) | (_, RuntimeValue::NaN) => Ok(RuntimeValue::NaN),

        (_, RuntimeValue::Number(0.0)) => Err(RuntimeError::DivisionByZero),

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(number_result(n / m)),

        // Vector scaling
        (RuntimeValue::Vector(v, ty), RuntimeValue::Number(scale)) => Ok(RuntimeValue::Vector(
            v.into_iter()
                .map(|value| eval_bin_div(value, RuntimeValue::Number(scale)))
                .collect::<Result<Vec<_>, _>>()?,
            ty,
        )),

        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn eval_bin_mod(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::NaN, _) | (_, RuntimeValue::NaN) => Ok(RuntimeValue::NaN),

        // x % 0
        (_, RuntimeValue::Number(0.0)) => Err(RuntimeError::ModuloByZero),

        // Infinity % anything
        (RuntimeValue::Infinity, _) | (RuntimeValue::NegInfinity, _) => Ok(RuntimeValue::NaN),

        // number % number
        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(number_result(n % m)),

        // vector % number
        (RuntimeValue::Vector(v, ty), RuntimeValue::Number(div)) => Ok(RuntimeValue::Vector(
            v.into_iter()
                .map(|value| eval_bin_mod(value, RuntimeValue::Number(div)))
                .collect::<Result<Vec<_>, _>>()?,
            ty,
        )),

        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn eval_bin_pow(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::NaN, _) | (_, RuntimeValue::NaN) => Ok(RuntimeValue::NaN),

        // number ^ number
        (RuntimeValue::Number(n), RuntimeValue::Number(exp)) => Ok(number_result(n.powf(exp))),

        (RuntimeValue::Infinity, RuntimeValue::Infinity) => Ok(RuntimeValue::Infinity),

        (RuntimeValue::Infinity, RuntimeValue::NegInfinity) => Ok(RuntimeValue::Number(0.0)),

        (RuntimeValue::NegInfinity, RuntimeValue::Infinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::NegInfinity) => Ok(RuntimeValue::NaN),

        // vector ^ number
        (RuntimeValue::Vector(v, ty), RuntimeValue::Number(exp)) => Ok(RuntimeValue::Vector(
            v.into_iter()
                .map(|value| eval_bin_pow(value, RuntimeValue::Number(exp)))
                .collect::<Result<Vec<_>, _>>()?,
            ty,
        )),

        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn is_truthy(x: RuntimeValue) -> bool {
    match x {
        RuntimeValue::Bool(false) | RuntimeValue::NaN | RuntimeValue::Null => false,
        _ => true,
    }
}

pub fn eval_bin_and(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => Ok(RuntimeValue::Bool(a && b)),
        (a, b) => Ok(RuntimeValue::Bool(is_truthy(a) && is_truthy(b))),
    }
}

pub fn eval_bin_or(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => Ok(RuntimeValue::Bool(a || b)),

        (a, b) => Ok(RuntimeValue::Bool(is_truthy(a) || is_truthy(b))),
    }
}

pub fn eval_bin_xor(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => Ok(RuntimeValue::Bool(a ^ b)),

        (a, b) => Ok(RuntimeValue::Bool(is_truthy(a) ^ is_truthy(b))),
    }
}

pub fn eval_bin_eq(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    if x.runtime_type() == y.runtime_type() {
        Ok(RuntimeValue::Bool(x.equals(&y)))
    } else {
        Err(RuntimeError::TypeMismatch {
            expected: x.runtime_type().stringify(),
            found: y.runtime_type().stringify(),
        })
    }
}

pub fn eval_bin_neq(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    if x.runtime_type() == y.runtime_type() {
        Ok(RuntimeValue::Bool(!x.equals(&y)))
    } else {
        Err(RuntimeError::TypeMismatch {
            expected: x.runtime_type().stringify(),
            found: y.runtime_type().stringify(),
        })
    }
}

pub fn eval_bin_ls(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Number(a), RuntimeValue::Number(b)) => match (a.fract(), b.fract()) {
            (0.0, 0.0) => Ok(number_result((a as i32 * 2_i32.pow(b as u32)) as f64)),
            _ => Err(RuntimeError::UnexpectedFract),
        },
        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn eval_bin_rs(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Number(a), RuntimeValue::Number(b)) => match (a.fract(), b.fract()) {
            (0.0, 0.0) => Ok(number_result((a as i32 / 2_i32.pow(b as u32)) as f64)),
            _ => Err(RuntimeError::UnexpectedFract),
        },
        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn eval_bin_cmp(x: RuntimeValue, y: RuntimeValue, mode: &str) -> RuntimeValueResult {
    match (x, y) {
        (RuntimeValue::Number(a), RuntimeValue::Number(b)) => Ok(RuntimeValue::Bool(match mode {
            "lt" => a < b,
            "le" => a <= b,
            "gt" => a > b,
            "ge" => a >= b,
            _ => false,
        })),
        _ => Err(RuntimeError::InvalidOperand),
    }
}
