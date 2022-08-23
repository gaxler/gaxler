use lang::{ConstIdx, OpCode};

use crate::Value;

#[derive(Debug, Clone)]
pub struct Chunk {
    ops: Vec<OpCode>,
    pub consts: Vec<Value>,
    /// source code line that got the opcode from
    pub line_nums: Vec<usize>,
}

#[inline]
fn op_comp(a: OpCode, b: OpCode) -> bool {
    std::mem::discriminant(&a) == std::mem::discriminant(&b)
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

    pub fn read_op(&self, ip: usize) -> Option<&OpCode> {
        self.ops.get(ip)
    }

    pub fn read_const(&self, addr: ConstIdx) -> &Value {
        &self.consts[addr as usize]
    }

    pub fn patch_op(&mut self, op: OpCode, ip: usize) {
        if !self.ops_match(op, ip) {
            let top = self.ops[ip];
            let msg = format!("Trying to patch unmatchin ops {:?}, {:?}", op, top);
            panic!("{}", &msg);
        }
        self.ops[ip] = op;
    }

    fn ops_match(&self, op: OpCode, ip: usize) -> bool {
        let inner_op = *self
            .read_op(ip)
            .expect("Read from wrong instruction addres");
        op_comp(op, inner_op)
    }

    pub fn patch_multip_op(&mut self, op: OpCode, ips: &[usize]) {
        for ip in ips {
            self.patch_op(op, *ip)
        }
    }

    pub fn debug_ops_dump(&self) {
        println!("\t OPS: {:?}", self.ops);
        println!("\t CONSTS: {:?}", self.consts);
    }

    pub fn get_line_num(&self, num: usize) -> usize {
        self.line_nums[num]
    }
}
