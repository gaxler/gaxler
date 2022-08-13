
use std::{cell::RefCell, mem::MaybeUninit};

use crate::{
    errors::{RTError, RuntimeError},
    opcode::{Chunk, OpCode}, value::{Value},
};

const STACK_MAX: usize = 256;

pub fn disassemble_op(chunk: &Chunk, offset: usize) {
    use OpCode::*;
    print!("{:04} ", offset);
    let op = &chunk.ops[offset];
    let line = &chunk.line_nums[offset];
    if offset > 0 && line == &chunk.line_nums[offset - 1] {
        print!("  |   ");
    } else {
        print!(" {:04} ", &line)
    }
    match op {
        RETURN => println!("OP: RETURN"),
        CONSTANT(idx) => {
            let const_val = chunk.consts[*idx as usize].clone();
            println!("OP: CONSTANT ({})", const_val)
        }
        t => {
            println!("OP: {:?}", t);
        }
    }
}

pub struct Stack {
    stack: [Value; STACK_MAX],
    top: usize,
}

impl Stack {
    fn init() -> Self {
        
        
         let empty_stack  = unsafe {

            let mut tmp  = MaybeUninit::<[Value; STACK_MAX]>::uninit().assume_init();

            for v in tmp.iter_mut() {
                *v = Value::Nil;
            }
            tmp
        };


        Self {
            stack:empty_stack, 
            top: 0,
        }
    }

    pub fn push(&mut self, val: Value) -> RTError<()> {
        if self.top > self.stack.len() - 1 {
            return Err(RuntimeError::StackError("Oveflow".to_string()));
        }

        self.stack[self.top] = val;
        self.top += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> RTError<Value> {
        if self.top == 0 {
            return Err(RuntimeError::StackError("Underflow".to_string()));
        }

        self.top -= 1;
        let pidx = self.top;
        let out_val = std::mem::replace(&mut self.stack[pidx], Value::Nil);
        Ok(out_val)
    }
}

fn exec_unary(op: &OpCode, stack: &mut Stack) -> RTError<()> {
    use OpCode::*;

    let unary_inp = stack.pop()?;

    let unary_result = match op {
        NEGATE => match unary_inp {
            Value::Int(v) => Value::Int(-v),
            Value::Float(v) => Value::Float(-v),
            Value::Nil => Value::Nil,
            _ => return Err(RuntimeError::IllegalUnaryOp(*op, unary_inp)),
        },
        // Take a value out of the stack, and negate it.
        //  is defined on the value enum that should be
        NOT => match unary_inp {
            Value::Bool(b) => Value::Bool(!b),
            Value::Nil => Value::Bool(true),
            _ => return Err(RuntimeError::IllegalUnaryOp(*op, unary_inp)),
        },
        _ => panic!("Not a unary op!!"),
    };

    stack.push(unary_result)?;

    Ok(())
}

fn exec_binary(op: &OpCode, stack: &mut Stack) -> Result<(), RuntimeError> {
    use OpCode::*;

    let v2 = stack.pop()?;
    let v1 = stack.pop()?;
    
    // clone those is cheap 
    let dbg_vals = (v1.clone(), v2.clone());
    

    let res = match op {
        ADD => v1.add(v2),
        SUB => v1.sub(v2),
        MUL => v1.mul(v2),
        DIV => v1.div(v2),
        EQUAL => v1.eq(v2),
        GREATER => v1.greater(v2),
        LESS => v2.greater(v1),
        _ => panic!("Non Binary!!"),
    };

    if let Value::Nil = res {
        let(v1, v2) = dbg_vals;
        let dbg_v1 = format!("{:?}", v1);
        let dbg_v2 = format!("{:?}", v2);
        return Err(RuntimeError::IllegalOp(*op, dbg_v1, dbg_v2));
    }

    stack.push(res)?;
    Ok(())
}

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize, // instruction pointer
    stack: RefCell<Stack>,
    // heap: RefMut<'a, Heap>,
    // stack: [Value; STACK_MAX],
    // stack_top: usize,
    debug: bool,
}

impl VM {
    pub fn init(debug: bool) -> Self {
        // let stack =[Value::Null; STACK_MAX];
        let stack = RefCell::new(Stack::init());
        Self {
            chunk: None,
            ip: 0,
            stack,
            // heap,
            debug,
        }
    }

    pub fn load_chunk(&mut self, chunk:Chunk) {
        self.ip = 0;
        self.chunk = Some(chunk);
    }

    pub fn unload_chunk(&mut self) -> Chunk {
        self.chunk.take().expect("Called unload on empty VM")
    }

    fn cur_chunk(&self) -> &Chunk {
        // TODO: produce proper runtime error. we might want to recover from this
        // but this thing is not gonna be used exept as a learing toy, so maybe no..
        self.chunk.as_ref().expect("Runtime Exception: Called run on an empty chunk")
    }

    fn read_byte(&self) -> &OpCode {
        // this can't be called outside of run, wehre we make sure that chunk is not empty
        let op = self.cur_chunk().read_op(self.ip);
        if self.debug {
            disassemble_op(self.cur_chunk(), self.ip);
        }
        op
    }

    pub fn push(&mut self, val: Value) {
        self.stack.borrow_mut().push(val).expect("Stack push error");
    }

    pub fn pop(&mut self) -> Value {
        self.stack.borrow_mut().pop().expect("Stack pop error")
    }

    pub fn run(&mut self) -> RTError<()> {
        use OpCode::*;

        loop {
            let op = self.read_byte();
            match op {
                RETURN => {
                    break;
                }
                CONSTANT(idx) => {
                    let val = self.cur_chunk().consts[*idx as usize].clone();
                    self.push(val);
                }
                NEGATE | NOT => {
                    let mut s = self.stack.borrow_mut();
                    exec_unary(op, &mut s).unwrap();
                }
                lit @ (NIL | FALSE | TRUE) => {
                    let val = Value::try_from(*lit);
                    if val.is_ok() {
                        self.push(val.unwrap());
                    }
                }

                ADD | SUB | MUL | DIV | LESS | GREATER | EQUAL => {
                    let mut s = self.stack.borrow_mut();
                    exec_binary(op, &mut s).unwrap();
                }
            }
            self.ip += 1;
        }
        if self.debug {
            let top = self.stack.borrow().top;
            let stack = self.stack.borrow().stack.to_vec();
            print!(" Stack: [ ");
            for (idx, s) in stack.iter().cloned().enumerate() {
                print!("{:?} ", s);
                if idx >= top {
                    break;
                }
            }
            println!(" ... ]");
        }
        // clear chunk
        Ok(())
    }

    pub fn show_stack(&self) {
        println!("Stack Values:");
        println!("==========");
        for (idx, v) in self.stack.borrow().stack.iter().enumerate() {
            match v {
                Value::Nil => continue,
                e => println!(" loc {} | {}", idx, *e)
            }
            
        }
        println!("==========");
    }
}
