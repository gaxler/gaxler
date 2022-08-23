use crate::errors::{COMPError, CompileError};

use values::Value;

use compiler::Compiler;
use lang::{ConstIdx, InstructAddr, OpCode};
use lang::{Precedence, Scanner, Token, TokenType};
use values::Chunk;

pub struct Parser<'a> {
    cur: Token,
    prev: Token,
    had_error: bool,
    panic_mode: bool,
    scanner: &'a mut Scanner<'a>,
    chunk: &'a mut Chunk,
    compiler: Compiler,
}

impl<'a> Parser<'a> {
    // pub fn init(scanner: &'a mut Scanner<'a>, chunk: &'a mut Chunk, heap:RefMut<'a, Heap>) -> Self {

    fn declaration(&mut self) -> COMPError<()> {
        match self.cur.ty {
            TokenType::Var => self.var_declaration()?,
            _ => self.statement()?,
        }
        Ok(())
    }

    fn statement(&mut self) -> COMPError<()> {
        match self.cur.ty {
            TokenType::Print => self.print()?,
            TokenType::If => self.if_else()?,
            TokenType::While => self.while_()?,
            TokenType::For => self.for_()?,
            TokenType::LeftBrace => self.scope()?,
            _ => self.expression_statement()?,
        }
        Ok(())
    }

    fn expression_statement(&mut self) -> COMPError<()> {
        self.expression(Precedence::None)?;
        self.cur_must_be(TokenType::Semicolon)?;
        self.emit_op(OpCode::POP);
        Ok(())
    }

    fn scope(&mut self) -> COMPError<()> {
        self.compiler.begin_scope();
        self.block()?;

        self.clean_locals();
        self.compiler.end_scope();

        Ok(())
    }

    fn block(&mut self) -> COMPError<()> {
        self.move_to_next_token();
        loop {
            match self.cur.ty {
                TokenType::RightBrace => break,
                TokenType::EoF => self.syntax_err("EoF without block close")?,
                _ => {
                    self.declaration()?;
                }
            }
        }
        self.cur_must_be(TokenType::RightBrace)?;
        Ok(())
    }

    fn expression(&mut self, min_prec: Precedence) -> COMPError<()> {
        use TokenType::*;

        // do the prefix op first
        // TODO: remove this and work on self.cur
        self.move_to_next_token();
        match self.prev.ty {
            Number => self.number()?,
            String => self.string()?,
            LeftParen => {
                self.expression(Precedence::None)?;
                self.cur_must_be(RightParen)?;
            }
            True | False | Nil => self.literal()?,
            Minus | Bang => self.unary()?,

            Ident => self.identifier()?,
            _ => self.syntax_err("Bad Expression")?,
        }

        // now do the infix and the res of those
        // if there is no infix operator here, we are done since the expression was handled

        // we just parser an expression, so we either have some infix op that
        // menas the expression continues or its something else. in this case we are done.

        loop {
            // let next_prec: Precedence = self.cur.ty.into();
            let next_prec = Precedence::from(self.cur.ty);
            if min_prec >= next_prec {
                // parse only stuff that has higher precedence than what we need
                return Ok(());
            }

            match self.cur.ty {
                Minus | Plus | Slash | Star | EqualEqual | BangEqual | Greater | GreaterEqual
                | LessEqual | Less | And | Or => self.binary()?,
                _ => break,
            }
        }
        Ok(())
    }

    fn var_declaration(&mut self) -> COMPError<()> {
        let ident_ = self.get_ident()?;

        // let var_name_const_idx = self.global_variable()?;

        if TokenType::Equal == self.cur.ty {
            self.move_to_next_token();
            // if we have an expresion that initializes the var, calculate it and put on stack
            self.expression(Precedence::None)?;
        } else {
            // do nothing, we just made room on out var table for this one and push NIL inside
            self.emit_op(OpCode::NIL);
        }

        if self.compiler.local_scope() {
            self.compiler
                .local_exists(&ident_)
                .expect("TODO: proper handling of double def of local variable ");
            self.compiler.add_local(ident_);
            // at this point the variable is already on the stack and is going to be used in the scope
            // it was deined in (or deeper scope)
        } else {
            let const_idx = self.chunk.add_const(Value::String(ident_.to_string())) as ConstIdx;
            self.emit_op(OpCode::DEFINE_GLOBAL(const_idx));
        }
        self.cur_must_be(TokenType::Semicolon)?;
        Ok(())
    }

    fn identifier(&mut self) -> COMPError<()> {
        let ident_ = self.scanner.token_text(self.prev)?;
        let is_local = self.compiler.find_local(&ident_);

        if let TokenType::Equal = self.cur.ty {
            self.move_to_next_token();
            self.expression(Precedence::None)?;
            match is_local {
                Some(slot) => self.emit_op(OpCode::SET_LOCAL(slot)),
                None => {
                    let ident_idx = self.chunk.add_const(Value::String(ident_)) as u8;
                    self.emit_op(OpCode::SET_GLOBAL(ident_idx))
                }
            }
        } else {
            match is_local {
                Some(slot) => self.emit_op(OpCode::GET_LOCAL(slot)),
                None => {
                    let ident_idx = self.chunk.add_const(Value::String(ident_)) as u8;
                    self.emit_op(OpCode::GET_GLOBAL(ident_idx))
                }
            }
        }
        Ok(())
    }

    fn literal(&mut self) -> COMPError<()> {
        match self.prev.ty {
            TokenType::True => self.emit_op(OpCode::TRUE),
            TokenType::False => self.emit_op(OpCode::FALSE),
            TokenType::Nil => self.emit_op(OpCode::NIL),
            _ => todo!(),
        }
        Ok(())
    }

    fn get_ident(&mut self) -> COMPError<String> {
        self.move_to_next_token();
        self.cur_must_be(TokenType::Ident)?;
        Ok(self.scanner.token_txt_str(self.prev)?.to_string())
    }

    fn _dbg(&self, tok: Token) {
        let a = CompileError::syntax(
            self.scanner.ascii_chars,
            "dbg",
            tok.start_pos,
            tok.start_pos + tok.len,
        );
        println!("{}", a);
    }

    fn print(&mut self) -> COMPError<()> {
        self.move_to_next_token();
        self.expression(Precedence::None)?;
        self.cur_must_be(TokenType::Semicolon)?;
        self.emit_op(OpCode::PRINT);
        Ok(())
    }

    fn for_(&mut self) -> COMPError<()> {
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

    fn while_(&mut self) -> COMPError<()> {
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

    fn if_else(&mut self) -> COMPError<()> {
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

    fn binary(&mut self) -> COMPError<()> {
        // at this point we already have the left hand side expression result on the stack
        // now we need to figure out what should we parse next. get that expression, push it on the stack
        // and do the binary operation on both the values in the stack.

        // the way bob does that in C is to have parse rules associated with every op token.
        // i can gothe same way as bob did maybe i should star with that and see how i can improve from there
        // I think it's much easir to do that with some pattern matching or something inplace. no need for extra functions here
        let op = self.cur.ty;
        self.move_to_next_token();

        match op {
            // do i need some kind of jump?
            TokenType::And => {
                let after_fst_expr_ip = self.chunk.count();
                self.emit_op(OpCode::JUMP_IF_FALSE(0xFFFF));
                self.expression(op.into())?;
                self.emit_op(OpCode::AND);
                let after_snd_expr_ip = self.chunk.count();
                self.chunk.patch_op(
                    OpCode::JUMP_IF_FALSE(after_snd_expr_ip as InstructAddr),
                    after_fst_expr_ip,
                );
            }
            TokenType::Or => {
                let fst_expr = self.chunk.count();
                self.emit_op(OpCode::JUMP_IF_FALSE(0xFFFF));
                let snd_expr = self.chunk.count();
                self.emit_op(OpCode::JUMP(0xFFFF));
                self.chunk
                    .patch_op(OpCode::JUMP_IF_FALSE(snd_expr as InstructAddr), fst_expr);
                self.expression(op.into())?;
                self.emit_op(OpCode::OR);
                let skip_snd_expr = self.chunk.count();
                self.chunk
                    .patch_op(OpCode::JUMP(skip_snd_expr as InstructAddr), snd_expr);
            }
            _ => {
                self.expression(op.into())?;
            }
        }

        // apply the binary op on both expressions
        match op {
            TokenType::Plus => self.emit_op(OpCode::ADD),
            TokenType::Minus => self.emit_op(OpCode::SUB),
            TokenType::Star => self.emit_op(OpCode::MUL),
            TokenType::Slash => self.emit_op(OpCode::DIV),
            TokenType::EqualEqual => self.emit_op(OpCode::EQUAL),
            TokenType::BangEqual => {
                self.emit_op(OpCode::EQUAL);
                self.emit_op(OpCode::NOT);
            }
            TokenType::Greater => self.emit_op(OpCode::GREATER),
            TokenType::GreaterEqual => {
                self.emit_op(OpCode::LESS);
                self.emit_op(OpCode::NOT);
            }
            TokenType::Less => self.emit_op(OpCode::LESS),
            TokenType::LessEqual => {
                self.emit_op(OpCode::GREATER);
                self.emit_op(OpCode::NOT)
            }
            TokenType::And | TokenType::Or => {
                // handled in previous match
            }
            _ => {
                dbg!(self.prev, self.cur);
                todo!()
            }
        }

        // let rulle = get_parse_rule(op);
        Ok(())
    }

    fn unary(&mut self) -> COMPError<()> {
        let op = self.prev.ty;

        self.expression(Precedence::Unary)?;

        match op {
            TokenType::Minus => self.emit_op(OpCode::NEGATE),
            TokenType::Bang => self.emit_op(OpCode::NOT),
            _ => {}
        };

        Ok(())
    }

    fn string(&mut self) -> COMPError<()> {
        let tok_txt = self
            .scanner
            .token_text(self.prev)
            .map_err(|_| CompileError::NonASCIIChar)?;

        let const_idx = self.chunk.add_const(Value::String(tok_txt));
        self.emit_op(OpCode::CONSTANT(const_idx as u8));
        Ok(())
    }

    fn number(&mut self) -> COMPError<()> {
        // for now this thing is f32 only
        let tok_txt = self.scanner.token_text(self.prev)?;
        let num: f32 = tok_txt.parse().map_err(|_| CompileError::NonASCIIChar)?;

        let const_idx = self.chunk.add_const(num.into());
        if const_idx > (u8::MAX - 1) as usize {
            return Err(CompileError::ToManyConstants);
        }
        self.emit_op(OpCode::CONSTANT(const_idx as u8));
        Ok(())
    }

    fn move_to_next_token(&mut self) {
        match self.scanner.scan_token() {
            Ok(tok) => {
                self.prev = self.cur;
                self.cur = tok;
            }
            Err(e) => {
                dbg!(e);

                self.had_error = true;
                self.panic_mode = true;
            }
        }
    }

    fn cur_must_be(&mut self, ty: TokenType) -> COMPError<()> {
        if self.cur.ty == ty {
            self.move_to_next_token();
            Ok(())
        } else {
            Err(CompileError::unexpected(
                self.scanner.ascii_chars,
                self.cur.ty,
                ty,
                self.scanner.start_pos,
                self.scanner.cur_pos,
            ))
        }
    }

    fn emit_op(&mut self, op: OpCode) {
        self.chunk.add_op(op, self.scanner.line as usize);
    }

    pub fn init(scanner: &'a mut Scanner<'a>, chunk: &'a mut Chunk) -> Self {
        let compiler = Compiler::init();

        let parser = Self {
            cur: Token::empty(0),
            prev: Token::empty(0),
            had_error: false,
            panic_mode: false,
            scanner,
            chunk,
            compiler,
        };
        parser
    }

    pub fn parse(&mut self) -> COMPError<()> {
        // we got a scanner and a chunk, now it's time to start writing
        self.move_to_next_token(); // get the first token for now ignore errors
                                   //for now we only want to cath an expression
        while self.cur.ty != TokenType::EoF {
            self.declaration()?;
            // self.expression(Precedence::Assignme"nt).unwrap();
        }

        self.cur_must_be(TokenType::EoF)?; // finished reading the whole scanner
        self.emit_op(OpCode::RETURN);
        Ok(())
    }

    fn syntax_err(&self, msg: &str) -> COMPError<()> {
        Err(CompileError::syntax(
            self.scanner.ascii_chars,
            msg,
            self.cur.start_pos,
            self.cur.start_pos + self.cur.len,
        ))
    }

    fn clean_locals(&mut self) {
        while self.compiler.should_pop_local() {
            self.emit_op(OpCode::POP);
        }
    }
}
