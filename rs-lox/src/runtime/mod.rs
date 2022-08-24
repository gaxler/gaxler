mod eval_loop;
mod ops;
mod utils;

use std::cell::RefCell;

use crate::errors::{RTError, RuntimeError};

use lang::{ConstIdx, OpCode};
use values::Value;
use values::{Chunk, Stack, VarStore};

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize, // instruction pointer
    stack: RefCell<Stack>,
    globals: VarStore,
    debug: bool,
}

impl VM {
    pub fn init(debug: bool) -> Self {
        // let stack =[Value::Null; STACK_MAX];
        let stack = RefCell::new(Stack::init());
        let globals = VarStore::new();
        Self {
            chunk: None,
            ip: 0,
            stack,
            globals,
            debug,
        }
    }
}
