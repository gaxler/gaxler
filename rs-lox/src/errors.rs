pub type COMPError<T> = Result<T, CompileError>;

pub type RTError<T> = Result<T, RuntimeError>;
use thiserror::Error;

pub use lang::CompileError;
use lang::OpCode;
use values::Value;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Stack: {0}")]
    StackError(String),
    #[error("Op {0:?} not allowed on {1:?} type")]
    IllegalUnaryOp(OpCode, Value),
    #[error("Op {0:?} not allowed on types {1:?} and {2:?}")]
    IllegalOp(OpCode, String, String),
    #[error("Unknown variable {0}")]
    UnknownVariable(String),
}
