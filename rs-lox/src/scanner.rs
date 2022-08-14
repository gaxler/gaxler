use std::{
    char,
    iter::{Enumerate, Peekable},
    str::Chars,
    string::FromUtf8Error,
};

use crate::errors::{COMPError, CompileError};


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
pub struct Token {
    pub ty: TokenType,
    pub start_pos: usize,
    pub len: usize,
    pub line: u32,
}

impl Token {
    fn make(token_type: TokenType, scanner: &Scanner) -> Self {
        let (start_pos, len) = match token_type {
           TokenType::String => {
            (scanner.start_pos+1, scanner.cur_pos-scanner.start_pos-2)   
           } 
           _ => (scanner.start_pos, scanner.cur_pos -scanner.start_pos)
        };
        
        Self {
            ty: token_type,
            start_pos, 
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

pub struct Scanner<'a> {
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
    Matched(Token),
}

macro_rules! next_is_or {
    ($ch:literal, $match_ty:ident, $else_ty:ident, $scanner:ident) => {
        if $scanner.next_is($ch) {
            $scanner.cur_pos += 1;
            $scanner.chars.next();
            MatchState::Matched($scanner._tok($match_ty))
        } else {
            MatchState::Matched($scanner._tok($else_ty))
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

    pub fn scan_token(&mut self) -> COMPError<Token> {
        use TokenType::*;

        self.start_pos = self.cur_pos;

        if self.cur_pos >= self.ascii_chars.len() {
            return Ok(self._tok(EoF));
        }

        'eval_loop: loop {
            match self.move_to_next_char() {
                None => return Ok(self._tok(EoF)),
                Some((_, ch)) => {
                    match self._token_matcher(ch) {
                        MatchState::Matched(tok) => return Ok(tok),
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
                            //Placeholder for error handling
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

    pub fn token_text(&self, tok: Token) -> COMPError<String> {
        let chars = self.ascii_chars[tok.start_pos..(tok.start_pos + tok.len)].to_vec();
        String::from_utf8(chars).map_err(|_| CompileError::NonASCIIChar)
    }


    fn _tok(&self, tok_type: TokenType) -> Token {
        Token::make(tok_type, &self)
    }

    fn next_is(&mut self, ch: char) -> bool {
        let next_ = self.chars.peek();
        if next_.is_none() {
            return false;
        }

        let (_, actual) = self.chars.peek().unwrap();
        *actual == ch
    }

    fn keyword_or_ident(&self, st: usize, en: usize) -> Token {
        
        // the shortest keyword is 2 chars
        if en - st < 1 {
            return self._tok(TokenType::Ident);
        }

        let first_char = self.ascii_chars[st] as char;
        let second_char = self.ascii_chars[st+1] as char;

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
            'f' => match second_char {
                'u' => match_rest("un", TokenType::Fun),
                'o' => match_rest("or", TokenType::For),
                'a' => match_rest("alse", TokenType::False),
                _ => self._tok(TokenType::Ident),
            },
            't' => match second_char {
                'h' => match_rest("his", TokenType::This),
                'r' => match_rest("rue", TokenType::True),
                _ => self._tok(TokenType::Ident),
            },

            _ => self._tok(TokenType::Ident),
        }
    }

    fn _token_matcher(&mut self, ch: char) -> MatchState {
        use TokenType::*;
        use MatchState::*;

        match ch {
            '0'..='9' => {
                while let Some(&(_, '0'..='9' | '.')) = self.peek() {
                    self.move_to_next_char();
                }
                Matched(self._tok(Number))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                while let Some(&(_, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')) = self.peek() {
                    self.move_to_next_char();
                }
                // this is the place we need to check and see what kind of string do we got here
                Matched(self.keyword_or_ident(self.start_pos, self.cur_pos))
            }
            '(' => Matched(self._tok(LeftParen)),
            ')' => Matched(self._tok(RightParen)),
            '{' => Matched(self._tok(LeftBrace)),
            '}' => Matched(self._tok(RightBrace)),
            ';' => Matched(self._tok(Semicolon)),
            ',' => Matched(self._tok(Comma)),
            '.' => Matched(self._tok(Dot)),
            '-' => Matched(self._tok(Minus)),
            '+' => Matched(self._tok(Plus)),
            '/' => {
                if self.next_is('/') {
                    while !self.next_is('\n') {
                        if self.move_to_next_char().is_none() {
                            return Matched(self._tok(EoF));
                        }
                    }
                    ScanNext
                } else {
                    Matched(self._tok(Slash))
                }
            }
            '*' => Matched(self._tok(Star)),
            '!' => next_is_or!('=', BangEqual, Bang, self),
            '=' => next_is_or!('=', EqualEqual, Equal, self),
            '<' => next_is_or!('=', LessEqual, Less, self),
            '>' => next_is_or!('=', GreaterEqual, Greater, self),
            // '"' => MatchState::StringLit,
            '"' => {
                while !self.next_is('"') {
                    match self.move_to_next_char() {
                        None => return SyntaxError,
                        Some((_, '\n')) => {
                            self.line += 1;
                        }
                        _ => {}
                    }
                }
                self.move_to_next_char();
                Matched(self._tok(String))
            }
            '\n' => ScanNextLine,
            ' ' | '\t' => ScanNext,
            _ => SyntaxError,
        }
    }

    fn move_to_next_char(&mut self) -> Option<(usize, char)> {
        let next_ = self.chars.next();
        if next_.is_some() {
            self.cur_pos += 1;
        }
        next_
    }

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
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
                let tok_str = scanner
                    .token_text(tok)
                    .map_err(|_| CompileError::NonASCIIChar)?;
                println!(" {:?} {}", tok.ty, tok_str);
            }
        }
        line = tok.line;
    }
    Ok(())
}
