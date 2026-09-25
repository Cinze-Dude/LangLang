use crate::runtime::values::{RuntimeType, RuntimeValue};

pub type RuntimeResult<T> = Result<T, RuntimeError>;
pub type RuntimeValueResult = RuntimeResult<RuntimeValue>;
pub type RuntimeTypeResult = RuntimeResult<RuntimeType>;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    UndefinedVariable(String),
    InvalidVariableName(String),
    InvalidVariableTarget(String),
    VariableTargetNameMismatch { expected: String, found: String },
    VariableNameAlreadyExists(String),
    OperationOnUndefinedValue,
    OperationImmutableValue(String),
    OperationDynamicValue(String),
    DivisionByZero,
    ModuloByZero,
    FactorialOverflow,
    FactorialNeg,
    UnexpectedFract,

    TypeMismatch { expected: String, found: String },

    IndexOutOfBounds,
    InvalidOperand,
    NotCallable,
    NumberError,
    InfinityError,
    TupleLengthOrTypeMismatch,

    InvalidType,

    NotImplemented,

    UndefinedFunctionName(String),
    ArgumentLengthMismatch { expected: String, found: String },

    Custom(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedFunctionName(name) => {
                write!(f, "undefined function name {}", name)
            }

            RuntimeError::ArgumentLengthMismatch { expected, found } => {
                write!(
                    f,
                    "argument length mismatch, expected '{}', found '{}'",
                    expected, found
                )
            }

            RuntimeError::VariableNameAlreadyExists(name) => {
                write!(f, "variable name already exists {}", name)
            }

            RuntimeError::TupleLengthOrTypeMismatch => {
                write!(f, "tuple length or type mismatch")
            }

            RuntimeError::UndefinedVariable(name) => {
                write!(f, "undefined variable '{}'", name)
            }

            RuntimeError::OperationOnUndefinedValue => {
                write!(f, "operation on undefined value")
            }

            RuntimeError::OperationImmutableValue(name) => {
                write!(f, "operation on immutable variable '{}'", name)
            }

            RuntimeError::OperationDynamicValue(name) => {
                write!(f, "operation on dynamic variable '{}'", name)
            }

            RuntimeError::InvalidVariableTarget(str) => {
                write!(f, "invalid variable target '{}'", str)
            }

            RuntimeError::VariableTargetNameMismatch { expected, found } => {
                write!(
                    f,
                    "variable target name mismatch: expected '{}', found '{}'",
                    expected, found
                )
            }

            RuntimeError::InvalidVariableName(name) => {
                write!(f, "invalid variable name '{}'", name)
            }

            RuntimeError::DivisionByZero => {
                write!(f, "division by zero")
            }

            RuntimeError::ModuloByZero => {
                write!(f, "modulo by zero")
            }

            RuntimeError::TypeMismatch { expected, found } => {
                write!(f, "expected type '{}', found '{}'", expected, found)
            }

            RuntimeError::IndexOutOfBounds => {
                write!(f, "index out of bounds")
            }

            RuntimeError::InvalidOperand => {
                write!(f, "invalid operand")
            }

            RuntimeError::NotCallable => {
                write!(f, "value is not callable")
            }

            RuntimeError::NumberError => {
                write!(f, "number error")
            }

            RuntimeError::InfinityError => {
                write!(f, "infinity error")
            }

            RuntimeError::FactorialOverflow => {
                write!(f, "factorial overflow")
            }

            RuntimeError::FactorialNeg => {
                write!(f, "factorial negative")
            }

            RuntimeError::UnexpectedFract => {
                write!(f, "did not expect fraction")
            }

            RuntimeError::InvalidType => {
                write!(f, "invalid type")
            }

            RuntimeError::NotImplemented => {
                write!(f, "not implemented")
            }

            RuntimeError::Custom(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
