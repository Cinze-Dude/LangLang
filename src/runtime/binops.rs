use crate::runtime::{
    errors::{RuntimeError, RuntimeValueResult},
    values::RuntimeValue,
};

pub fn fact(num: i32) -> Result<i32, RuntimeError> {
    match num {
        0 => Ok(1),
        12.. => Err(RuntimeError::FactorialOverflow),
        1.. => Ok(fact(num - 1)? * num),
        _ => Err(RuntimeError::InvalidOperand),
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

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(RuntimeValue::Number(n + m)),

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

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(RuntimeValue::Number(n - m)),

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

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(RuntimeValue::Number(n * m)),

        // Vector scaling
        (RuntimeValue::Vector(v, ty), RuntimeValue::Number(scale)) => Ok(RuntimeValue::Vector(
            v.into_iter()
                .map(|value| eval_bin_div(value, RuntimeValue::Number(scale)))
                .collect::<Result<Vec<_>, _>>()?,
            ty,
        )),

        (RuntimeValue::Number(scale), RuntimeValue::Vector(v, ty)) => Ok(RuntimeValue::Vector(
            v.into_iter()
                .map(|value| eval_bin_div(value, RuntimeValue::Number(scale)))
                .collect::<Result<Vec<_>, _>>()?,
            ty,
        )),

        _ => Err(RuntimeError::InvalidOperand),
    }
}

pub fn eval_bin_div(x: RuntimeValue, y: RuntimeValue) -> RuntimeValueResult {
    match (x, y) {
        (_, RuntimeValue::Number(0.0)) => Err(RuntimeError::DivisionByZero),

        (RuntimeValue::Infinity, RuntimeValue::Infinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::NegInfinity)
        | (RuntimeValue::Infinity, RuntimeValue::NegInfinity)
        | (RuntimeValue::NegInfinity, RuntimeValue::Infinity) => Ok(RuntimeValue::NaN),

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

        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(RuntimeValue::Number(n / m)),

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
        (RuntimeValue::Number(n), RuntimeValue::Number(m)) => Ok(RuntimeValue::Number(n % m)),

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
        (RuntimeValue::Number(n), RuntimeValue::Number(exp)) => {
            Ok(RuntimeValue::Number(n.powf(exp)))
        }

        // infinity ^ positive number
        (RuntimeValue::Infinity, RuntimeValue::Number(exp)) => {
            if exp > 0.0 {
                Ok(RuntimeValue::Infinity)
            } else if exp == 0.0 {
                Ok(RuntimeValue::Number(1.0))
            } else {
                Ok(RuntimeValue::Number(0.0))
            }
        }

        // -infinity ^ integer
        (RuntimeValue::NegInfinity, RuntimeValue::Number(exp)) => {
            if exp == 0.0 {
                Ok(RuntimeValue::Number(1.0))
            } else if exp.fract() == 0.0 {
                if (exp as i64) % 2 == 0 {
                    Ok(RuntimeValue::Infinity)
                } else {
                    Ok(RuntimeValue::NegInfinity)
                }
            } else {
                Ok(RuntimeValue::NaN)
            }
        }

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
