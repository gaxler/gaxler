use super::*;

impl<'a> Parser<'a> {
    pub(super) fn for_(&mut self) -> COMPError<()> {
        self.move_to_next_token();
        self.compiler.begin_scope();
        self.cur_must_be(TokenType::LeftParen)?;

        let mut to_loop_body: Vec<usize> = vec![];
        let mut to_loop_end: Vec<usize> = vec![];

        match self.cur.ty {
            // this thing means there is no initializer
            TokenType::Semicolon => {}
            TokenType::Var => self.var_declaration()?,
            // assignment to seomthing declared
            _ => self.expression_statement()?,
        }
        let before_cond = self.chunk.count();

        if TokenType::Semicolon != self.cur.ty {
            self.expression(Precedence::None)?;
            self.cur_must_be(TokenType::Semicolon)?;

            // jump to the end of the loop
            to_loop_end.push(self.chunk.count());
            self.emit_op(OpCode::JUMP_IF_FALSE(0xFF));
            // no false, we get rid of the conditoinal (if we jump we will get rid of it in loop closure)
            self.emit_op(OpCode::POP);

            to_loop_body.push(self.chunk.count());
            self.emit_op(OpCode::JUMP(0xFF));
        }

        // if there is no increase clause this thing will be the same as loop body
        let mut inc_clause = before_cond;

        // increment clause. this one is tricky.
        // single pass parser.
        // we define it here, but it must run after the body is executed
        if self.cur.ty != TokenType::RightParen {
            to_loop_body.push(self.chunk.count());
            self.emit_op(OpCode::JUMP(0xFF));

            inc_clause = self.chunk.count();

            self.expression(Precedence::Assignment)?;
            self.emit_op(OpCode::POP);
            // this happens only if we have increase clause
            self.emit_op(OpCode::JUMP(before_cond as u16));
        }

        self.cur_must_be(TokenType::RightParen)?;
        let loop_body = self.chunk.count();
        self.chunk
            .patch_multip_op(OpCode::JUMP(loop_body as u16), &to_loop_body);
        self.statement()?;
        self.emit_op(OpCode::JUMP(inc_clause as u16));
        let loop_end = self.chunk.count();
        self.chunk
            .patch_multip_op(OpCode::JUMP_IF_FALSE(loop_end as u16), &to_loop_end);
        self.emit_op(OpCode::POP);
        self.clean_locals();
        self.compiler.end_scope();
        Ok(())
    }

    pub(super) fn while_(&mut self) -> COMPError<()> {
        self.move_to_next_token();
        self.cur_must_be(TokenType::LeftParen)?;
        let loop_start = self.chunk.count();
        self.expression(Precedence::None)?;
        // at this point we have some result on the stack
        self.cur_must_be(TokenType::RightParen)?;

        // if rhis result is flase we jumpt to the end of the loop
        let jmp_addr = self.chunk.count();
        self.emit_op(OpCode::JUMP_IF_FALSE(0xFFFF));
        // throw away the old loop condition from the stack
        self.emit_op(OpCode::POP);
        self.statement()?;
        self.emit_op(OpCode::JUMP(loop_start as u16));

        let end_loop = self.chunk.count();
        self.chunk
            .patch_op(OpCode::JUMP_IF_FALSE(end_loop as u16), jmp_addr);

        // in case we jumped to the end, we need to pop whatever we had in there
        self.emit_op(OpCode::POP);
        Ok(())
    }

    pub(super) fn if_else(&mut self) -> COMPError<()> {
        self.move_to_next_token();
        self.cur_must_be(TokenType::LeftParen)?;
        self.expression(Precedence::None)?;
        self.cur_must_be(TokenType::RightParen)?;

        // here need to read the conditioin an decide where to next
        // the should be a true block and a false block
        // the at the end of the true block there needs to be a jump to skip the else block
        // so i need a way to follow the chunk size
        // using Chunk len should do the trick
        let true_block_ip = self.chunk.count();
        self.emit_op(OpCode::JUMP_IF_FALSE(0xFFFF));
        // let before_then = dbg!((self.prev, self.cur));
        self.statement()?;

        // we need to patch something here, in case this is true
        let mut end_of_true = self.chunk.count();

        if self.cur.ty == TokenType::Else {
            self.move_to_next_token();
            self.emit_op(OpCode::JUMP(0xFFFF));
            // else statement
            self.statement()?;
            let end_of_false = self.chunk.count();
            // go to the jump op and fix it
            self.chunk
                .patch_op(OpCode::JUMP(end_of_false as InstructAddr), end_of_true);
            // add the extra op we got from the else clause jump
            end_of_true += 1;
        };

        self.chunk.patch_op(
            OpCode::JUMP_IF_FALSE(end_of_true as InstructAddr),
            true_block_ip,
        );
        Ok(())
    }
}
