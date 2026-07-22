use crate::runtime::values::RuntimeValue;

pub type RuntimeResult<T> = Result<T, RuntimeError>;
pub type RuntimeValueResult = RuntimeResult<RuntimeValue>;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    UndefinedVariable(String),
    DivisionByZero,
    TypeMismatch { expected: String, found: String },
    IndexOutOfBounds,
    InvalidOperand,
    NotCallable,
    NumberError,
    InfinityError,
    NotImplemented,
    Custom(String),
}
