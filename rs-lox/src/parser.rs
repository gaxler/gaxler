use crate::{
    errors::{COMPError, CompileError},
};

use values::Value;

use compiler::{Compiler, Local};
use lang::{ConstIdx, OpCode};
use lang::{Precedence, Scanner, Token, TokenType};
use values::Chunk;

pub struct Parser<'a> {
    cur: Token,
    prev: Token,
    had_error: bool,
    panic_mode: bool,
    scanner: &'a mut Scanner<'a>,
    chunk: &'a mut Chunk,
    compiler: Compiler<'a>,
}

impl<'a> Parser<'a> {
    // pub fn init(scanner: &'a mut Scanner<'a>, chunk: &'a mut Chunk, heap:RefMut<'a, Heap>) -> Self {

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

    fn declaration(&mut self) -> COMPError<()> {
        match self.cur.ty {
            TokenType::Var => {
                let var_name_const_idx = self.variable()?;

                if TokenType::Equal == self.cur.ty {
                    self.move_to_next_token();
                    // if we have an expresion that initializes the var, calculate it and put on stack
                    self.expression(Precedence::None)?;
                } else {
                    // do nothing, we just made room on out var table for this one and push NIL inside
                    self.emit_op(OpCode::NIL);
                }

                if self.compiler.local_scope() {
                    let ident_ = self.scanner.token_txt_str(self.prev)?;
                    self.compiler.local_exists(ident_).expect("TODO: proper handling of double def of local variable ");
                    self.compiler.add_local(ident_);
                    // at this point the variable is already on the stack and is going to be used in the scope
                    // it was deined in (or deeper scope) 
                } else {
                    self.emit_op(OpCode::DEFINE_GLOBAL(var_name_const_idx));
                }
                self.cur_must_be(TokenType::Semicolon)?;
            }
            _ => self.statement()?,
        }
        Ok(())
    }

    fn statement(&mut self) -> COMPError<()> {
        match self.cur.ty {
            TokenType::Print => {
                self.move_to_next_token();
                self.expression(Precedence::None)?;
                self.cur_must_be(TokenType::Semicolon)?;
                self.emit_op(OpCode::PRINT);
            }
            TokenType::LeftBrace => {
                self.compiler.begin_scope();
                self.block()?;

                while self.compiler.should_pop_local() {
                    self.emit_op(OpCode::POP);
                }
                self.compiler.end_scope();
            }
            // Expression statement
            _ => {
                self.expression(Precedence::None)?;
                self.cur_must_be(TokenType::Semicolon)?;
                self.emit_op(OpCode::POP);
            }
        }
        Ok(())
    }

    fn block(&mut self) -> COMPError<()> {
        self.move_to_next_token();
        loop {
            match self.cur.ty {
                TokenType::RightBrace => break,
                TokenType::EoF => {
                    return Err(CompileError::syntax(
                        self.scanner.ascii_chars,
                        "EoF without block close",
                        self.scanner.start_pos,
                        self.scanner.cur_pos,
                    ));
                }

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
        self.move_to_next_token();

        match self.prev.ty {
            Number => self.number()?,
            String => self.string()?,
            LeftParen => {
                self.expression(Precedence::None)?;
                self.cur_must_be(RightParen)?;
            }
            True | False | Nil => {
                self.literal()?;
            }
            Minus | Bang => self.unary()?,

            Ident => {
                let ident_ = self.scanner.token_text(self.prev)?;
                let ident_idx = self.chunk.add_const(Value::String(ident_));

                if let TokenType::Equal = self.cur.ty {
                    self.move_to_next_token();
                    self.expression(Precedence::None)?;
                    self.emit_op(OpCode::SET_GLOBAL(ident_idx as u8))
                } else {
                    self.emit_op(OpCode::GET_GLOBAL(ident_idx as u8));
                }
            }

            _ => {
                // Expression that doesn't start with a prefix op or a literal is poorly formed
                return Err(CompileError::syntax(
                    self.scanner.ascii_chars,
                    "Bad expression",
                    self.scanner.start_pos,
                    self.scanner.cur_pos,
                ));
            }
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
                | LessEqual | Less => {
                    self.move_to_next_token();
                    self.binary()?
                }
                _ => break,
            }
        }
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

    fn literal(&mut self) -> COMPError<()> {
        match self.prev.ty {
            TokenType::True => self.emit_op(OpCode::TRUE),
            TokenType::False => self.emit_op(OpCode::FALSE),
            TokenType::Nil => self.emit_op(OpCode::NIL),
            _ => todo!(),
        }
        Ok(())
    }

    fn variable(&mut self) -> COMPError<ConstIdx> {
        self.move_to_next_token();
        self.cur_must_be(TokenType::Ident)?;
        let ident_ = self.scanner.token_text(self.prev)?;
        let const_idx = self.chunk.add_const(Value::String(ident_)) as ConstIdx;
        Ok(const_idx)
    }

    fn binary(&mut self) -> COMPError<()> {
        // at this point we already have the left hand side expression result on the stack
        // now we need to figure out what should we parse next. get that expression, push it on the stack
        // and do the binary operation on both the values in the stack.

        // the way bob does that in C is to have parse rules associated with every op token.
        // i can gothe same way as bob did maybe i should star with that and see how i can improve from there
        // I think it's much easir to do that with some pattern matching or something inplace. no need for extra functions here
        let op = self.prev.ty;

        // put RHS expression on the stach
        self.expression(op.into())?;

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
}
