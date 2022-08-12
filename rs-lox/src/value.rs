use std::{fmt, hash::Hash};

use crate::opcode::OpCode;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i32),
    Float(f32),
    String(HeapPtr),
}

type HeapPtr = usize;

pub struct Heap {
    heap: Vec<Obj>,
}

impl Heap {
    fn put(&mut self, obj: Obj) -> HeapPtr {
        self.heap.push(obj);
        self.heap.len()
    }

    fn get(&self, addr: HeapPtr) -> Option<&Obj> {
        self.heap.get(addr)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub enum Obj {
    String(String),
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
            _ => return Err(()),
        };
        Ok(res)
    }
}

macro_rules! map_expr {
    ($self:ident->$v1:ident, $other:ident->$v2:ident, $e:expr, ($($val_ty:ident),*)) => {
        match ($self, $other) {
            $(
                (Value::$val_ty($v1), Value::$val_ty($v2)) => Value::$val_ty($e),
            )*
            _ => Value::Nil
        }
    };
    // different binary functions per type with different output types
    (
        $self:expr => $v1:ident,
        $other:expr => $v2:ident,

            $(($e:expr, [$($ty:ident),*] -> $out_ty:ident)),*


    ) => {
        match ($self, $other) {
            $(
                $(
                (Value::$ty($v1), Value::$ty($v2)) => Value::$out_ty($e),
                )*
            )*
            _ => Value::Nil
        }
    }
}

impl Value {
    pub fn add(&self, other: Self) -> Self {
        map_expr!(self->v1, other->v2, v1+v2, (Int, Float, String))

        // value_op!(self->v1, other->v2, v1+v2)
    }

    pub fn sub(&self, other: Self) -> Self {
        map_expr!(self->v1, other->v2, v1-v2, (Int, Float))
    }

    pub fn mul(&self, other: Self) -> Self {
        map_expr!(self->v1, other->v2, v1*v2, (Int, Float))
    }
    pub fn div(&self, other: Self) -> Self {
        map_expr!(self->v1, other->v2, v1/v2, (Int, Float))
    }

    pub fn eq(&self, other: Self) -> Self {
        map_expr!(*self => v1, other => v2, 
            (v1==v2,[Int, Float] -> Bool),
            (!(v1^v2), [Bool] -> Bool)
        )
        // map_expr!(&self => v1, other => v2, (v1==v2,[Int, Float]), (v1^v2, [Bool]))
        // use Value::*;
        // let res = match (self, other) {
        //     (Int(v1), Int(v2)) => *v1 == v2,
        //     (Float(v1), Float(v2)) => *v1 == v2,
        //     (Bool(b1), Bool(b2)) => !((*b1) ^ b2),
        //     (Nil, Nil) => true,
        //     _ => return Nil,
        // };
        // Bool(res)
    }

    pub fn greater(&self, other: Self) -> Self {
        map_expr!(*self => v1, other => v2, 
            (v1 > v2,[Int, Float] -> Bool),
            (v1 && v1^v2, [Bool] -> Bool)
        )

        // use Value::*;
        // let res = match (*self, other) {
        //     (Int(v1), Int(v2)) => v1 > v2,
        //     (Float(v1), Float(v2)) => v1 > v2,
        //     (Bool(b1), Bool(b2)) => b1 && b1 ^ b2,
        //     (Nil, Nil) => true,
        //     _ => return Nil,
        // };
        // Bool(res)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "String@{:04}", v),
            Value::Nil => write!(f, "Nil"),
        }
    }
}
