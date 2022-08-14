use std::collections::HashMap;

use crate::value::Value;

pub type ConstIdx = u8;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum OpCode {
    RETURN,
    CONSTANT(ConstIdx), // load the constant to the vm for use
    NEGATE,
    NOT,
    NIL,
    TRUE,

    FALSE,
    EQUAL,
    LESS,
    GREATER,
    ADD,
    SUB,
    MUL,
    DIV,

    PRINT,
    POP,
    DEFINE_GLOBAL(ConstIdx),
    GET_GLOBAL(ConstIdx),
    SET_GLOBAL(ConstIdx),
}


pub struct Chunk {
    pub ops: Vec<OpCode>,
    pub consts: Vec<Value>,
    /// source code line that got the opcode from
    pub line_nums: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            ops: vec![],
            consts: vec![],
            line_nums: vec![],
        }
    }

    pub fn count(&self) -> usize {
        self.ops.len()
    }

    /// This adds an OpCode to our code chunk
    /// unlike in C, we don't need to handle the growth and size counter for a vec. its already a part of std::Vec
    pub fn add_op(&mut self, op: OpCode, line: usize) {
        self.ops.push(op);
        self.line_nums.push(line);
    }

    /// Returns the offset to the constant array of the latest constant added
    pub fn add_const(&mut self, val: Value) -> usize {
        self.consts.push(val);
        self.consts.len() - 1
    }

    pub fn read_op(&self, ip: usize) -> &OpCode {
        &self.ops[ip]
    }

    pub fn read_const(&self, addr: ConstIdx) -> &Value {
        &self.consts[addr as usize]
    }
}


#[derive(Debug, Clone)]
pub struct VarStore {
    store: HashMap<String, Value>
}


impl VarStore {
    pub fn new() -> Self {
        let store = HashMap::new();
        Self {store}
    }

    pub fn put(&mut self, ident_: String, val: Value) {
        self.store.insert(ident_, val);
    }

    pub fn get(&self, ident_: &str) -> Option<&Value> {
        self.store.get(ident_)
    }

    pub fn contains(&self, ident_: &str) -> bool {
        self.store.contains_key(ident_)
    }
}