use std::{fmt, str::FromStr};

use lang::OpCode;

#[derive(Clone, Debug)]
#[repr(u8)]
/// Represents dynamic values in Lox.
/// Every value can have several types. Also, large types like classes and string are stored on the heap as heap objects.
/// we loose the ability to make this a copy type since we have heap object. and i want them to have the drop trait
pub enum Value {
    Nil,
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String), // String(HeapPtr),
                    // Obj(HeapObj)
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

impl TryFrom<Value> for String {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(s) = value {
            return Ok(s);
        }
        Err(())
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

impl TryFrom<Value> for bool {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            Value::Nil => Ok(false),
            _ => Err(()),
        }
    }
}

macro_rules! map_expr {
    ($v1:ident <= $self:expr , $v2:ident <= $other:expr, $e:expr, ($($val_ty:ident),*)) => {
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
    };
    // different binary functions per type with same output types
    (
        $self:expr => $v1:ident,
        $other:expr => $v2:ident,

            $(($e:expr, [$($ty:ident),*])),*


    ) => {
        match ($self, $other) {
            $(
                $(
                (Value::$ty($v1), Value::$ty($v2)) => Value::$ty($e),
                )*
            )*
            _ => Value::Nil
        }
    }
}

#[inline]
fn _add_str_slices(s1: &str, s2: &str) -> Value {
    let mut s3 = String::from_str(s1).unwrap();
    s3.push_str(s2);
    Value::String(s3)
}

impl Value {
    pub fn add(&self, other: Self) -> Self {
        use Value::*;
        match (self, other) {
            (Int(v1), Int(v2)) => Int(v1 + v2),
            (Float(v1), Float(v2)) => Float(v1 + v2),
            (String(v1), Float(v2)) => {
                let fstr = v2.to_string();
                _add_str_slices(v1.as_str(), &fstr)
            }
            (Float(v1), String(v2)) => {
                let fstr = v1.to_string();
                _add_str_slices(&fstr, v2.as_str())
            }
            (String(s1), String(s2)) => _add_str_slices(s1, &s2),
            _ => Nil,
        }
    }

    pub fn sub(&self, other: Self) -> Self {
        map_expr!(v1 <= self, v2 <= other, v1 - v2, (Int, Float))
    }

    pub fn mul(&self, other: Self) -> Self {
        map_expr!(v1 <= self, v2 <= other, v1 * v2, (Int, Float))
    }
    pub fn div(&self, other: Self) -> Self {
        map_expr!(v1 <= self, v2 <= other, v1 / v2, (Int, Float))
    }

    pub fn eq(&self, other: Self) -> Self {
        map_expr!(self => v1, other => v2,
            (*v1==v2,[Int, Float, String] -> Bool),
            (!(v1^v2), [Bool] -> Bool)
        )
    }

    pub fn greater(&self, other: Self) -> Self {
        map_expr!(self => v1, other => v2,
            (*v1 > v2,[Int, Float] -> Bool),
            (*v1 && v1^v2, [Bool] -> Bool)
        )
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Nil => write!(f, "Nil"),
        }
    }
}
