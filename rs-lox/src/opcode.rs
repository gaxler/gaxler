 use std::fmt;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum OpCode {
    RETURN,
    CONSTANT(u8), // load the constant to the vm for use
    NEGATE,
    NOT,
    NIL,
    TRUE,
    FALSE,
    ADD,
    SUB,
    MUL,
    DIV
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Value {
    Nil,
    Bool(bool),
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

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl TryFrom<OpCode> for Value {

    type Error = ();

    fn try_from(value: OpCode) -> Result<Self, Self::Error> {
        let res = match value {
            OpCode::NIL => Value::Nil,
            OpCode::TRUE => Value::Bool(true),
            OpCode::FALSE => Value::Bool(false),
            _ => return Err(())   
        };
        Ok(res)
    }
    
}


macro_rules! value_op {
    ($self:ident->$v1: ident, $other:ident->$v2:ident, $e:expr) => {
        match ($self, $other) {
            (Value::Int($v1), Value::Int($v2)) => Value::Int($e),
            (Value::Float($v1), Value::Float($v2)) => Value::Float($e),
            _ => Value::Nil
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
            Value::Bool(v) => write!(f, "{}", v),
            Value::Nil => write!(f, "Nil")
            
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
