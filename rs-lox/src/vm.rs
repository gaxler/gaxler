use std::{borrow::Borrow, cell::RefCell, mem::MaybeUninit};

use crate::{
    errors::{COMPError, RTError, RuntimeError},
    opcode::{Chunk, ConstIdx, OpCode, VarStore},
    value::Value,
};

const STACK_MAX: usize = 256;

pub struct Stack {
    stack: [Value; STACK_MAX],
    top: usize,
}

impl Stack {
    fn init() -> Self {
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

    pub fn peek(&self) -> Option<&Value> {
        if self.top == 0 {
            return None;
        }

        Some(self.stack[self.top - 1].borrow())
    }
}

fn exec_unary(op: OpCode, stack: &mut Stack) -> RTError<()> {
    use OpCode::*;

    let unary_inp = stack.pop()?;

    let unary_result = match op {
        NEGATE => match unary_inp {
            Value::Int(v) => Value::Int(-v),
            Value::Float(v) => Value::Float(-v),
            Value::Nil => Value::Nil,
            _ => return Err(RuntimeError::IllegalUnaryOp(op, unary_inp)),
        },
        // Take a value out of the stack, and negate it.
        //  is defined on the value enum that should be
        NOT => match unary_inp {
            Value::Bool(b) => Value::Bool(!b),
            Value::Nil => Value::Bool(true),
            _ => return Err(RuntimeError::IllegalUnaryOp(op, unary_inp)),
        },
        _ => panic!("Not a unary op!!"),
    };

    stack.push(unary_result)?;

    Ok(())
}

fn exec_binary(op: OpCode, stack: &mut Stack) -> Result<(), RuntimeError> {
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
        let (v1, v2) = dbg_vals;
        let dbg_v1 = format!("{:?}", v1);
        let dbg_v2 = format!("{:?}", v2);
        return Err(RuntimeError::IllegalOp(op, dbg_v1, dbg_v2));
    }

    stack.push(res)?;
    Ok(())
}

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize, // instruction pointer
    stack: RefCell<Stack>,
    globals: VarStore,
    debug: bool,
}

impl VM {
    pub fn init(debug: bool) -> Self {
        // let stack =[Value::Null; STACK_MAX];
        let stack = RefCell::new(Stack::init());
        let globals = VarStore::new();
        Self {
            chunk: None,
            ip: 0,
            stack,
            globals,
            debug,
        }
    }

    pub fn run(&mut self) -> RTError<()> {
        use OpCode::*;

        loop {
            let op = self.read_byte();
            match op {
                NIL => {
                    self.push(Value::Nil);
                }
                RETURN => {
                    break;
                }
                CONSTANT(idx) => {
                    let val = self.cur_chunk().read_const(idx).clone();
                    self.push(val);
                }
                NEGATE | NOT => {
                    let mut s = self.stack.borrow_mut();
                    exec_unary(op, &mut s).unwrap();
                }
                lit @ (NIL | FALSE | TRUE) => {
                    let val = Value::try_from(lit);
                    if val.is_ok() {
                        self.push(val.unwrap());
                    }
                }

                ADD | SUB | MUL | DIV | LESS | GREATER | EQUAL => {
                    let mut s = self.stack.borrow_mut();
                    exec_binary(op, &mut s).unwrap();
                }

                PRINT => {
                    println!("{}", self.pop())
                }
                POP => {
                    self.pop();
                }
                DEFINE_GLOBAL(ident_idx) => {
                    // got in trouble with the slices. they don't live long enough (according to borrow checker).
                    // need some interning mechanism...
                    let key = self._read_ident(ident_idx);
                    let val = self.pop();
                    self.globals.put(key, val);
                }
                GET_GLOBAL(ident_idx) => {
                    let key = self._read_ident(ident_idx);
                    let val = self.globals.get(&key);
                    if val.is_none() {
                        self.debug_dump();
                        return Err(RuntimeError::UnknownVariable(key));
                    }
                    let val = val.unwrap().clone();
                    self.push(val);
                }
                SET_GLOBAL(ident_idx) => {
                    let ident_ = self._read_ident(ident_idx);
                    if !self.globals.contains(&ident_) {
                        return Err(RuntimeError::UnknownVariable(ident_));
                    }

                    self.globals.put(ident_, self.peek()?);
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

    pub fn load_chunk(&mut self, chunk: Chunk) {
        self.ip = 0;
        self.chunk = Some(chunk);
    }

    pub fn unload_chunk(&mut self) -> Chunk {
        self.chunk.take().expect("Called unload on empty VM")
    }

    fn cur_chunk(&self) -> &Chunk {
        // TODO: produce proper runtime error. we might want to recover from this
        // but this thing is not gonna be used exept as a learing toy, so maybe no..
        self.chunk
            .as_ref()
            .expect("Runtime Exception: Called run on an empty chunk")
    }

    fn read_byte(&self) -> OpCode {
        // this can't be called outside of run, wehre we make sure that chunk is not empty
        let op = *self.cur_chunk().read_op(self.ip);
        if self.debug {
            disassemble_op(self.cur_chunk(), self.ip);
        }
        op
    }

    fn push(&mut self, val: Value) {
        self.stack.borrow_mut().push(val).expect("Stack push error");
        if self.debug {
            print!("After Push: ");
            self.show_stack();
        }
    }

    fn pop(&mut self) -> Value {
        if self.debug {
            print!("Before Pop: ");
            self.show_stack();
        }

        match self.stack.borrow_mut().pop() {
            Ok(v) => v,
            Err(e) => {
                println!(" [ Error -> @{:04}:{} ]", self.ip, e);
                panic!()
            }
        }
    }

    fn peek(&self) -> RTError<Value> {
        self.stack
            .borrow()
            .peek()
            .map(|v| v.clone())
            .ok_or(RuntimeError::StackError(
                "Peeked on an empty staclk".to_string(),
            ))
    }

    pub fn show_stack(&self) {
        println!("Stack Values:");
        println!("==========");
        for (idx, v) in self.stack.borrow().stack.iter().enumerate() {
            match v {
                Value::Nil => continue,
                e => println!(" loc {} | {}", idx, *e),
            }
        }
        println!("==========");
    }

    fn _read_ident(&self, ident_: ConstIdx) -> String {
        let key: String = self
            .cur_chunk()
            .read_const(ident_)
            .clone()
            .try_into()
            .expect("Found an identifier that is not a string");
        key
    }

    fn debug_dump(&mut self) {
        if let Some(chunk) = self.chunk.take() {
            println!(" ===== Constants =====");
            for cons in chunk.consts {
                println!(" -> {}", cons);
            }
        };
        println!("===== Globals ======");
        println!(" -> {:?}", self.globals);
    }
}

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
        CONSTANT(idx) => {
            let const_val = chunk.consts[*idx as usize].clone();
            println!("OP: CONSTANT ({})", const_val)
        }
        t => {
            println!("OP: {:?}", t);
        }
    }
}
