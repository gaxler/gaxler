use super::*;

use super::ops::{exec_binary, exec_unary};

impl VM {
    pub fn run(&mut self) -> RTError<()> {
        use OpCode::*;

        loop {
            let op = self.read_byte();
            if self.debug {
                println!(
                    "\t {}) {:?} <- line {}",
                    self.ip,
                    op,
                    self.cur_chunk().line_nums[self.ip]
                )
            };
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

            if self.debug {
                print!("\t");
                self.show_stack();
            };
            self.ip += 1;
        }
        // clear chunk
        Ok(())
    }
}
