use lang::{OpCode, ConstIdx};

use crate::Value;

#[derive(Debug, Clone)]
pub struct Chunk {
    ops: Vec<OpCode>,
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
    
    pub fn patch_op(&mut self, op: OpCode, ip: usize) {
        self.ops[ip] = op;
    }

    pub fn debug_ops_dump(&self) {
        println!("{:?}", self.ops);
    }
}
