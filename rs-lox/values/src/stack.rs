use std::{mem::MaybeUninit, borrow::Borrow};

use crate::Value;

const STACK_MAX: usize = u8::MAX as usize + 1;

pub enum StackError {
    Underflow,
    Overflow
}

pub struct Stack {
    stack: [Value; STACK_MAX],
    top: usize,
}

impl Stack {
    fn init() -> Self {
        
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
}
