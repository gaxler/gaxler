 use std::fmt;

#[derive(Debug)]
#[repr(u8)]
pub enum OpCode {
    RETURN,
    CONSTANT(u8), // load the constant to the vm for use
    NEGATE,
    ADD,
    SUB,
    MUL,
    DIV
}

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Null,
    Int(i32),  
    Float(f32),
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}


impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Int(v)
    }
}

macro_rules! value_op {
    ($self:ident->$v1: ident, $other:ident->$v2:ident, $e:expr) => {
        match ($self, $other) {
            (Value::Int($v1), Value::Int($v2)) => Value::Int($e),
            (Value::Float($v1), Value::Float($v2)) => Value::Float($e),
            _ => Value::Null
        }   
    };
}

impl Value {

    pub fn add(&self, other: Self) -> Self {
        value_op!(self->v1, other->v2, v1+v2)
    }

    pub fn sub(&self, other: Self) -> Self {
        value_op!(self->v1, other->v2, v1-v2)
    }

    pub fn mul(&self, other: Self) -> Self {
        value_op!(self->v1, other->v2, v1*v2)
    } 
    pub fn div(&self, other: Self) -> Self {
        value_op!(self->v1, other->v2, v1/v2)
    } 
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Null => write!(f, "NULL")
            
        }
    }
}


pub struct Chunk {
    pub ops: Vec<OpCode>,
    pub consts: Vec<Value>,
    /// source code line that got the opcode from
    pub line_nums: Vec<usize>
}


impl Chunk {
    pub fn new() -> Self{
        Self {ops:vec![], consts: vec![], line_nums: vec![]}
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

}

