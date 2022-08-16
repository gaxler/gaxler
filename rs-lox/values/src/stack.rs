use std::{borrow::Borrow, mem::MaybeUninit};

use crate::Value;
use thiserror::Error;

const STACK_MAX: usize = u8::MAX as usize + 1;

#[derive(Debug, Error)]
pub enum StackError {
    #[error("Stack Underflow")]
    Underflow,
    #[error("Stack Overflow")]
    Overflow,
}

pub struct Stack {
    stack: [Value; STACK_MAX],
    top: usize,
}

impl Stack {
    pub fn init() -> Self {
        // SAFETY: init an array with non-copy value.
        let empty_stack = unsafe {
            let mut tmp = MaybeUninit::<[Value; STACK_MAX]>::uninit().assume_init();

            for v in tmp.iter_mut() {
                *v = Value::Nil;
            }
            tmp
        };

        Self {
            stack: empty_stack,
            top: 0,
        }
    }

    pub fn push(&mut self, val: Value) -> Result<(), StackError> {
        if self.top > self.stack.len() - 1 {
            return Err(StackError::Overflow);
        }

        self.stack[self.top] = val;
        self.top += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Value, StackError> {
        if self.top <= 0 {
            return Err(StackError::Underflow);
        }

        self.top -= 1;
        let pidx = self.top;
        let out_val = std::mem::replace(&mut self.stack[pidx], Value::Nil);
        Ok(out_val)
    }

    pub fn peek(&self) -> Option<&Value> {
        if self.top == 0 {
            return None;
        }

        Some(self.stack[self.top - 1].borrow())
    }

    pub fn show_stack(&self) {
        println!("Stack Values:");
        println!("==========");
        for (idx, v) in self.stack.iter().enumerate() {
            match v {
                Value::Nil => continue,
                e => println!(" loc {} | {}", idx, *e),
            }
        }
        println!("==========");
    }
}

impl std::fmt::Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();

        let top = self.top;
        let stack = self.stack.to_vec();
        res.push_str(" Stack: [ ");
        for (idx, s) in stack.iter().cloned().enumerate() {
            res.push_str(&format!("{:?} ", s));
            if idx >= top {
                break;
            }
        }
        res.push_str(" ... ]");
        res.push('\n');
        write!(f, "{}", res)
    }
}
