use crate::{scanner::{TokenType, Token, Scanner}, opcode::{Chunk, OpCode}};

type COMPError<T> = Result<T, CompileError>;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Source code must be ASCII chars only")]
    NonASCIIChar,
    #[error("Syntax error on line {line} at char {ch}")]
    SyntaxError { line: u32, ch: usize },
    #[error("Expected Token {0:?} found Token {1:?}")]
    UnexpectedToken(TokenType, TokenType),
    #[error("Constant is indexed by u8")]
    ToManyConstants,
}


#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment, // =
    Or,
    And,
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // !, -
    Call,       // . f()
    Primary,
}

impl From<TokenType> for Precedence {
    fn from(ty: TokenType) -> Self {
        use TokenType::*;

        match ty {
            Equal => Self::Assignment,
            Or => Self::Or,
            And => Self::And,
            EqualEqual | BangEqual => Self::Equality,
            Greater | GreaterEqual | Less | LessEqual => Self::Comparison,
            Plus | Minus => Self::Term, // what happens in unary setting with minus?
            Star | Slash => Self::Factor,
            Dot => Self::Call,
            _ => Self::None,
        }
    }
}

struct Parser<'a> {
    cur: Token,
    prev: Token,
    had_error: bool,
    panic_mode: bool,
    scanner: &'a mut Scanner<'a>,
    chunk: &'a mut Chunk,
}

impl<'a> Parser<'a> {
    fn init(scanner: &'a mut Scanner<'a>, chunk: &'a mut Chunk) -> Self {
        let parser = Self {
            cur: Token::empty(0),
            prev: Token::empty(0),
            had_error: false,
            panic_mode: false,
            scanner,
            chunk,
        };
        parser
    }

    fn advance(&mut self) {
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

    fn expect_token(&mut self, ty: TokenType) -> COMPError<()> {
        if self.cur.ty == ty {
            self.advance();
            Ok(())
        } else {
            Err(CompileError::UnexpectedToken(ty, self.cur.ty))
        }
    }

    fn emit_op(&mut self, op: OpCode) {
        self.chunk.add_op(op, self.scanner.line as usize);
    }

    fn expression(&mut self, min_prec: Precedence) -> COMPError<()> {
        use TokenType::*;

        // do the prefix op first
        self.advance();

        match self.prev.ty {
            Number => self.number()?,
            LeftParen => {
                self.expression(Precedence::None)?;
                self.expect_token(TokenType::RightParen)?;
            },
            True | False | Nil => {
                self.literal()?;
            }
            Minus => self.unary()?,

            _ => {
                // Expression that doesn't start with a prefix op or a literal is poorly formed
                return Err(CompileError::SyntaxError {
                    line: self.scanner.line,
                    ch: self.scanner.start_pos,
                });
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

            self.advance();

            match self.prev.ty {
                Minus | Plus | Slash | Star => self.binary()?,
                _ => break
            }
        }
        Ok(())
    }

    fn literal(&mut self) -> COMPError<()> {
        match self.prev.ty {
            TokenType::True => self.emit_op(OpCode::TRUE),
            TokenType::False => self.emit_op(OpCode::FALSE),
            TokenType::Nil => self.emit_op(OpCode::NIL),
            _ => todo!()
            
        }
        Ok(())
    }

    fn binary(&mut self) -> COMPError<()> {
        // at this point we already have the left hand side expression result on the stack
        // now we need to figure out what should we parse next. get that expression, push it on the stack
        // and do the binary operation on both the values in the stack.

        // the way bob does that in C is to have parse rules associated with every op token.
        // i can gothe same way as bob did maybe i should star with that and see how i can improve from there
        // I think it's much easir to do that with some pattern matching or something inplace. no need for extra functions here
        let op = self.prev.ty;
        self.expression(op.into())?;

        match op {
            TokenType::Plus => self.emit_op(OpCode::ADD),
            TokenType::Minus => self.emit_op(OpCode::SUB),
            TokenType::Star => self.emit_op(OpCode::MUL),
            TokenType::Slash => self.emit_op(OpCode::DIV),
            _ => {
                dbg!(self.prev, self.cur);
                todo!()
            },
        }

        // let rulle = get_parse_rule(op);
        Ok(())
    }

    fn unary(&mut self) -> COMPError<()> {
        let op = self.prev.ty;

        self.expression(Precedence::Unary)?;

        match op {
            TokenType::Minus => self.emit_op(OpCode::NEGATE),
            _ => {}
        };

        Ok(())
    }

    fn number(&mut self) -> COMPError<()> {
        // for now this thing is f32 only
        let tok_txt= self.scanner.token_text(self.prev).map_err(|_| CompileError::NonASCIIChar)?;
        let num: f32 = tok_txt
            .parse()
            .map_err(|_| CompileError::NonASCIIChar)?;

        let const_idx = self.chunk.add_const(num.into());
        if const_idx > (u8::MAX - 1) as usize {
            return Err(CompileError::ToManyConstants);
        }
        self.emit_op(OpCode::CONSTANT(const_idx as u8));
        Ok(())
    }

    fn grouping(&mut self) -> COMPError<()> {
        self.expression(Precedence::None)?;
        self.expect_token(TokenType::RightParen)?;
        Ok(())
    }

    fn parse(&mut self) {
        // we got a scanner and a chunk, now it's time to start writing
        self.advance(); // get the first token for now ignore errors
                        //for now we only want to cath an expression

        while self.cur.ty != TokenType::EoF {
            dbg!(self.cur.ty);
            self.expression(Precedence::Assignment).unwrap();
        }

        self.expect_token(TokenType::EoF).unwrap(); // finished reading the whole scanner
        self.emit_op(OpCode::RETURN);
    }
}
