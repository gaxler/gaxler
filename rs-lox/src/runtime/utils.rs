use super::*;

pub(super) fn stack_pop(stack: &mut Stack) -> RTError<Value> {
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

pub(super) fn stack_push(val: Value, stack: &mut Stack) -> RTError<()> {
    stack
        .push(val)
        .map_err(|e| RuntimeError::StackError(format!("{}", e)))
}

impl VM {
    pub fn load_chunk(&mut self, chunk: Chunk) {
        self.ip = 0;
        self.chunk = Some(chunk);
    }

    pub fn unload_chunk(&mut self) -> Chunk {
        self.chunk.take().expect("Called unload on empty VM")
    }

    pub(super) fn cur_chunk(&self) -> &Chunk {
        // TODO: produce proper runtime error. we might want to recover from this
        // but this thing is not gonna be used exept as a learing toy, so maybe no..
        self.chunk
            .as_ref()
            .expect("Runtime Exception: Called run on an empty chunk")
    }

    pub(super) fn read_byte(&self) -> OpCode {
        // this can't be called outside of run, wehre we make sure that chunk is not empty
        if self.ip >= self.cur_chunk().count() {
            self.cur_chunk().debug_ops_dump();
            print!("\t STACK: ");
            self.show_stack();
        }
        let op = *self.cur_chunk().read_op(self.ip).expect(&format!(
            " IP Out of Bounds: Failed to read Op from {} instruction pointer",
            self.ip
        ));
        if self.debug {
            // disassemble_op(self.cur_chunk(), self.ip);
        }
        op
    }

    pub(super) fn push(&mut self, val: Value) {
        self.stack.borrow_mut().push(val).expect("Stack push error");
    }

    pub(super) fn pop(&mut self) -> Value {
        match self.stack.borrow_mut().pop() {
            Ok(v) => v,
            Err(e) => {
                println!(" [ Error -> @{:04}:{} ]", self.ip, e);
                panic!()
            }
        }
    }

    pub(super) fn peek(&self) -> RTError<Value> {
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

    pub(super) fn _read_ident(&self, ident_: ConstIdx) -> String {
        let key: String = self
            .cur_chunk()
            .read_const(ident_)
            .clone()
            .try_into()
            .expect("Found an identifier that is not a string");
        key
    }

    pub(super) fn debug_dump(&mut self) {
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
