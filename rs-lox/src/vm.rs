use std::cell::RefCell;

use crate::errors::{RTError, RuntimeError};

use lang::{ConstIdx, OpCode};
use values::Value;
use values::{Chunk, Stack, VarStore};

fn stack_pop(stack: &mut Stack) -> RTError<Value> {
    let res = stack
        .pop()
        .map_err(|e| RuntimeError::StackError(format!("{}", e)));
    match res {
        Err(e) => {
            stack.show_stack();
            println!("Got in trouble for {}", e);
            Err(e)
        }
        Ok(v) => Ok(v),
    }
}

fn stack_push(val: Value, stack: &mut Stack) -> RTError<()> {
    stack
        .push(val)
        .map_err(|e| RuntimeError::StackError(format!("{}", e)))
}

fn exec_unary(op: OpCode, stack: &mut Stack) -> RTError<()> {
    use OpCode::*;

    let unary_inp = stack_pop(stack)?;

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

    stack_push(unary_result, stack)?;

    Ok(())
}

fn exec_binary(op: OpCode, stack: &mut Stack) -> Result<(), RuntimeError> {
    use OpCode::*;
    let v2 = stack_pop(stack)?;
    let v1 = stack_pop(stack)?;

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
        AND => v1.and(v2),
        OR => v1.or(v2),
        _ => panic!("Non Binary!!"),
    };

    if let Value::Nil = res {
        let (v1, v2) = dbg_vals;
        let dbg_v1 = format!("{:?}", v1);
        let dbg_v2 = format!("{:?}", v2);
        return Err(RuntimeError::IllegalOp(op, dbg_v1, dbg_v2));
    }

    stack
        .push(res)
        .map_err(|e| RuntimeError::StackError(format!("{}", e)))?;
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
            if self.debug {println!("\t {}) {:?} <- line {}", self.ip, op, self.cur_chunk().line_nums[self.ip])};
            match op {
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

                ADD | SUB | MUL | DIV | LESS | GREATER | EQUAL | AND | OR => {
                    let mut s = self.stack.borrow_mut();

                    exec_binary(op, &mut s).expect(&format!(
                        "Failed on line {}",
                        self.cur_chunk().get_line_num(self.ip)
                    ));
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
                GET_LOCAL(slot) => {
                    // expressions leave stuff on the stack, but we don't allow naked expression anymore
                    // variable declaration stay in the stack. so we either have variables, or we are in the middle of an expression.
                    // in that case any changes to the stack happen after the locals and doesn't affect locals oreder.
                    let val = self
                        .stack
                        .borrow_mut()
                        .peek_at(slot as usize)
                        .expect("Local Slot in invalid stack location???")
                        .clone();
                    self.push(val);
                }
                SET_GLOBAL(ident_idx) => {
                    let ident_ = self._read_ident(ident_idx);
                    if !self.globals.contains(&ident_) {
                        return Err(RuntimeError::UnknownVariable(ident_));
                    }

                    self.globals.put(ident_, self.peek()?);
                }
                SET_LOCAL(slot) => {
                    // we see equal after an identifier, we evaluate and expression (result on stack) and call the assignemnt OP
                    let val = self.peek()?;
                    *self
                        .stack
                        .borrow_mut()
                        .peek_at(slot as usize)
                        .expect("Local Slot in invalid stack location???") = val;
                }
                JUMP_IF_FALSE(new_ip) => {

                    let last_val: bool = self
                        .peek()
                        .unwrap()
                        .try_into()
                        .expect("Trying to create  a bool from non bool Value");
                    if !last_val {
                        self.ip = new_ip as usize;
                        continue;
                    }
                }
                JUMP(new_ip) => {
                    self.ip = new_ip as usize;
                    continue;
                }
            }
            
            if self.debug { print!("\t"); self.show_stack()};
            self.ip += 1;
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
        if self.ip >= self.cur_chunk().count() {
            self.cur_chunk().debug_ops_dump();
        }
        let op = *self.cur_chunk().read_op(self.ip);
        if self.debug {
            // disassemble_op(self.cur_chunk(), self.ip);
        }
        op
    }

    fn push(&mut self, val: Value) {
        self.stack.borrow_mut().push(val).expect("Stack push error");
    }

    fn pop(&mut self) -> Value {
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
        self.stack.borrow().show_stack();
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
        if let Some(chunk) = self.chunk.clone() {
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
    let op = chunk.read_op(offset);
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
