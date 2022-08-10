use std::{
    char,
    iter::{Enumerate, Peekable},
    str::Chars, string::FromUtf8Error,
};

use thiserror::Error;

use crate::opcode::{Chunk, OpCode};
type COMPError<T> = Result<T, CompileError>;

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

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Ident,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    EoF,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Token {
    pub ty: TokenType,
    pub start_pos: usize,
    pub len: usize,
    pub line: u32,
}

impl Token {
    fn make(token_type: TokenType, scanner: &Scanner) -> Self {
        let len = scanner.cur_pos - scanner.start_pos;
        Self {
            ty: token_type,
            start_pos: scanner.start_pos,
            len,
            line: scanner.line,
        }
    }

    pub fn empty(line: u32) -> Self {
        Self {
            ty: TokenType::Error,
            start_pos: 0,
            len: 0,
            line,
        }
    }
}

pub(crate) struct Scanner<'a> {
    ascii_chars: &'a [u8],
    chars: Peekable<Enumerate<Chars<'a>>>,
    pub start_pos: usize,
    pub cur_pos: usize,
    pub line: u32,
}

enum MatchState {
    SyntaxError,
    ScanNext,
    ScanNextLine,
    Match(Token),
}

macro_rules! match_next {
    ($scanner:ident, $ch:literal, $match_ty:ident, $else_ty:ident) => {
        if $scanner.match_next($ch) {
            $scanner.cur_pos += 1;
            $scanner.chars.next();
            MatchState::Match($scanner._tok($match_ty))
        } else {
            MatchState::Match($scanner._tok($else_ty))
        }
    };
}

impl<'a> Scanner<'a> {
    pub fn from_str(source: &'a str) -> COMPError<Self> {
        if !source.is_ascii() {
            return Err(CompileError::NonASCIIChar);
        }
        let ascii_chars = source.as_bytes();
        Ok(Self {
            ascii_chars,
            start_pos: 0,
            cur_pos: 0,
            line: 1,
            chars: source.chars().enumerate().peekable(),
        })
    }

    fn _tok(&self, tok_type: TokenType) -> Token {
        Token::make(tok_type, &self)
    }

    fn match_next(&mut self, ch: char) -> bool {
        let next_ = self.chars.peek();
        if next_.is_none() {
            return false;
        }

        let (_, actual) = self.chars.peek().unwrap();
        *actual == ch
    }

    fn match_keywords(&self, st: usize, en: usize) -> Token {
        let first_char = self.ascii_chars[st] as char;
        let match_rest = |tgt: &str, kw: TokenType| {
            if tgt
                .as_bytes()
                .iter()
                .eq(self.ascii_chars[st + 1..en].into_iter())
            {
                self._tok(kw)
            } else {
                self._tok(TokenType::Ident)
            }
        };

        match first_char {
            'a' => match_rest("nd", TokenType::And),
            'c' => match_rest("lass", TokenType::Class),
            'e' => match_rest("lse", TokenType::Else),
            'i' => match_rest("f", TokenType::If),
            'n' => match_rest("il", TokenType::Nil),
            'o' => match_rest("r", TokenType::Or),
            'p' => match_rest("rint", TokenType::Print),
            'r' => match_rest("eturn", TokenType::Return),
            's' => match_rest("uper", TokenType::Super),
            'v' => match_rest("ar", TokenType::Var),
            'w' => match_rest("hile", TokenType::While),
            'f' => match self.ascii_chars[st + 1] as char {
                'u' => match_rest("un", TokenType::Fun),
                'o' => match_rest("or", TokenType::For),
                'a' => match_rest("alse", TokenType::False),
                _ => self._tok(TokenType::Ident),
            },
            't' => match self.ascii_chars[st + 1] as char {
                'h' => match_rest("his", TokenType::This),
                'r' => match_rest("rue", TokenType::True),
                _ => self._tok(TokenType::Ident),
            },

            _ => self._tok(TokenType::Ident),
        }
    }

    fn _token_matcher(&mut self, ch: char) -> MatchState {
        use TokenType::*;

        match ch {
            '0'..='9' => {
                while let Some(&(_, '0'..='9' | '.')) = self.peek() {
                    self.advance();
                }
                MatchState::Match(self._tok(Number))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                while let Some(&(_, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')) = self.peek() {
                    self.advance();
                }
                // this is the place we need to check and see what kind of string do we got here
                MatchState::Match(self.match_keywords(self.start_pos, self.cur_pos))
            }
            '(' => MatchState::Match(self._tok(LeftParen)),
            ')' => MatchState::Match(self._tok(RightParen)),
            '{' => MatchState::Match(self._tok(LeftBrace)),
            '}' => MatchState::Match(self._tok(RightBrace)),
            ';' => MatchState::Match(self._tok(Semicolon)),
            ',' => MatchState::Match(self._tok(Comma)),
            '.' => MatchState::Match(self._tok(Dot)),
            '-' => MatchState::Match(self._tok(Minus)),
            '+' => MatchState::Match(self._tok(Plus)),
            '/' => {
                if self.match_next('/') {
                    while !self.match_next('\n') {
                        if self.advance().is_none() {
                            return MatchState::Match(self._tok(EoF));
                        }
                    }
                    MatchState::ScanNext
                } else {
                    MatchState::Match(self._tok(Slash))
                }
            }
            '*' => MatchState::Match(self._tok(Star)),
            '!' => match_next!(self, '=', BangEqual, Bang),
            '=' => match_next!(self, '=', EqualEqual, Equal),
            '<' => match_next!(self, '=', LessEqual, Less),
            '>' => match_next!(self, '=', GreaterEqual, Greater),
            // '"' => MatchState::StringLit,
            '"' => {
                while !self.match_next('"') {
                    match self.advance() {
                        None => return MatchState::SyntaxError,
                        Some((_, '\n')) => {
                            self.line += 1;
                        }
                        _ => {}
                    }
                }
                self.advance();
                MatchState::Match(self._tok(TokenType::String))
            }
            '\n' => MatchState::ScanNextLine,
            ' ' | '\t' => MatchState::ScanNext,
            _ => MatchState::SyntaxError,
        }
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let next_ = self.chars.next();
        if next_.is_some() {
            self.cur_pos += 1;
        }
        next_
    }

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }

    pub fn scan_token(&mut self) -> COMPError<Token> {
        use TokenType::*;

        self.start_pos = self.cur_pos;

        if self.cur_pos >= self.ascii_chars.len() {
            return Ok(self._tok(EoF));
        }

        'eval_loop: loop {
            match self.advance() {
                None => return Ok(self._tok(EoF)),
                Some((_, ch)) => {
                    match self._token_matcher(ch) {
                        MatchState::Match(tok) => return Ok(tok),
                        MatchState::ScanNextLine => {
                            self.line += 1;
                            self.start_pos = self.cur_pos;
                            continue 'eval_loop;
                        }
                        MatchState::ScanNext => {
                            self.start_pos = self.cur_pos;
                            continue 'eval_loop;
                        }

                        MatchState::SyntaxError => {
                            // Place holder for error handling
                        }
                    }
                }
            }
            break;
        }
        Err(CompileError::SyntaxError {
            line: self.line,
            ch: self.start_pos,
        })
        // Token::make_error(self.line)
    }
    pub fn token_text(&self, tok: Token) -> Result<String, FromUtf8Error>{
        let chars = self.ascii_chars[tok.start_pos..(tok.start_pos+tok.len)].to_vec();
        String::from_utf8(chars)

    }
}

pub fn dummy_compile(source: &str) -> COMPError<()> {
    let mut scanner = Scanner::from_str(source)?;

    let mut line = 0u32;

    loop {
        let tok = scanner.scan_token().unwrap();

        if tok.line != line {
            print!("{:04} ", tok.line);
        } else {
            print!("  | ")
        }

        match tok.ty {
            TokenType::Error | TokenType::EoF => {
                println!(" {:?} ", tok.ty);
                break;
            }
            _ => {
                let tok_str = scanner.token_text(tok)?;
                println!(" {:?} {}", tok.ty, tok_str);
            }
        }
        line = tok.line;
    }
    Ok(())
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
        let tok_txt = self.prev.start_pos..(self.prev.start_pos + self.prev.len);
        let num: f32 = String::from_utf8(self.scanner.ascii_chars[tok_txt].to_ascii_lowercase())
            .unwrap()
            .parse()
            .unwrap();

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

pub fn compile(source: &str) -> Chunk {
    let mut scanner = Scanner::from_str(source).unwrap();
    let mut chunk = Chunk::new();

    let mut parser = Parser::init(&mut scanner, &mut chunk);
    parser.parse();

    chunk
}
