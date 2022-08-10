pub type COMPError<T> = Result<T, CompileError>;

pub type RTError<T> = Result<T, RuntimeError>;
use thiserror::Error;

use crate::{scanner::TokenType, opcode::{OpCode, Value}};

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Source code must be ASCII chars only")]
    NonASCIIChar,
    #[error("Syntax error on line {line} at char {ch}")]
    SyntaxError { line: u32, ch: usize },
    #[error("Expected Token {0:?} found Token {1:?}")]
    UnexpectedToken(TokenType, TokenType),
    #[error("Constant is indexed by u8")]
    ToManyConstants,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Stack: {0}")]
    StackError(String),
    #[error("Op {0:?} not allowed on {1:?} type")]
    IllegalUnaryOp(OpCode, Value),
    #[error("Op {0:?} not allowed on types {1:?} and {2:?}")]
    IllegalOp(OpCode, Value, Value),
}

