use std::{error, iter::Peekable, str::Chars};

use crate::{utils::cite_span, Token, TokenType};
use thiserror::Error;

type COMPError<T> = Result<T, CompileError>;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Source code must be ASCII chars only")]
    NonASCIIChar,
    #[error("Syntax Error: {0} \n\t {1}")]
    SyntaxError(String, String),
    #[error("Expected Token {0:?} found Token {1:?} \n {2}")]
    UnexpectedToken(TokenType, TokenType, String),
    #[error("Constant is indexed by u8")]
    ToManyConstants,
}

impl CompileError {
    fn prep_cite(source: &[u8], st_pos: usize, en_pos: usize) -> String {
        let src = String::from_utf8(source.to_vec()).expect("Bad Code string");
        cite_span(&src, st_pos, en_pos)
    }

    pub fn syntax(source: &[u8], msg: &str, st_pos: usize, en_pos: usize) -> Self {
        Self::SyntaxError(msg.to_string(), Self::prep_cite(source, st_pos, en_pos))
    }

    pub fn unexpected(
        source: &[u8],
        tok: TokenType,
        exp: TokenType,
        st_pos: usize,
        en_pos: usize,
    ) -> Self {
        let cite = Self::prep_cite(source, st_pos, en_pos);
        Self::UnexpectedToken(exp, tok, cite)
    }
}

pub struct Scanner<'a> {
    pub ascii_chars: &'a [u8],
    chars: Peekable<Chars<'a>>,
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
            MatchState::Matched($scanner.make_token($match_ty))
        } else {
            MatchState::Matched($scanner.make_token($else_ty))
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
            chars: source.chars().peekable(),
        })
    }

    pub fn scan_token(&mut self) -> COMPError<Token> {
        use TokenType::*;

        self.start_pos = self.cur_pos;

        if self.cur_pos >= self.ascii_chars.len() {
            return Ok(self.make_token(EoF));
        }

        'eval_loop: loop {
            match self.move_to_next_char() {
                None => return Ok(self.make_token(EoF)),

                Some(ch) => {
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
        Err(CompileError::syntax(
            self.ascii_chars,
            "Couldn't identify tokens",
            self.start_pos,
            self.cur_pos,
        ))
        // Token::make_error(self.line)
    }

    pub fn token_text(&self, tok: Token) -> COMPError<String> {
        let chars = self.ascii_chars[tok.start_pos..(tok.start_pos + tok.len)].to_vec();
        String::from_utf8(chars).map_err(|_| CompileError::NonASCIIChar)
    }

    pub fn token_txt_str(&self, tok: Token) -> COMPError<&'a str> {
        let chars = &self.ascii_chars[tok.start_pos..(tok.start_pos + tok.len)];
        std::str::from_utf8(chars).map_err(|_| CompileError::NonASCIIChar)
    }

    fn make_token(&self, tok_type: TokenType) -> Token {
        Token::make(tok_type, &self)
    }

    fn next_is(&mut self, ch: char) -> bool {
        let next_ = self.chars.peek();
        if next_.is_none() {
            return false;
        }

        let actual = self.chars.peek().unwrap();
        *actual == ch
    }

    fn keyword_or_ident(&self, st: usize, en: usize) -> Token {
        // the shortest keyword is 2 chars
        if en - st < 1 {
            return self.make_token(TokenType::Ident);
        }

        let first_char = self.ascii_chars[st] as char;
        let second_char = self.ascii_chars[st + 1] as char;

        let match_rest = |tgt: &str, kw: TokenType| {
            if tgt
                .as_bytes()
                .iter()
                .eq(self.ascii_chars[st + 1..en].into_iter())
            {
                self.make_token(kw)
            } else {
                self.make_token(TokenType::Ident)
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
                _ => self.make_token(TokenType::Ident),
            },
            't' => match second_char {
                'h' => match_rest("his", TokenType::This),
                'r' => match_rest("rue", TokenType::True),
                _ => self.make_token(TokenType::Ident),
            },

            _ => self.make_token(TokenType::Ident),
        }
    }

    fn _token_matcher(&mut self, ch: char) -> MatchState {
        use MatchState::*;
        use TokenType::*;

        match ch {
            '0'..='9' => {
                while let Some(&('0'..='9' | '.')) = self.peek() {
                    self.move_to_next_char();
                }
                Matched(self.make_token(Number))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                while let Some(&('a'..='z' | 'A'..='Z' | '_' | '0'..='9')) = self.peek() {
                    self.move_to_next_char();
                }
                // this is the place we need to check and see what kind of string do we got here
                Matched(self.keyword_or_ident(self.start_pos, self.cur_pos))
            }
            '(' => Matched(self.make_token(LeftParen)),
            ')' => Matched(self.make_token(RightParen)),
            '{' => Matched(self.make_token(LeftBrace)),
            '}' => Matched(self.make_token(RightBrace)),
            ';' => Matched(self.make_token(Semicolon)),
            ',' => Matched(self.make_token(Comma)),
            '.' => Matched(self.make_token(Dot)),
            '-' => Matched(self.make_token(Minus)),
            '+' => Matched(self.make_token(Plus)),
            '/' => {
                if self.next_is('/') {
                    while !self.next_is('\n') {
                        if self.move_to_next_char().is_none() {
                            return Matched(self.make_token(EoF));
                        }
                    }
                    ScanNext
                } else {
                    Matched(self.make_token(Slash))
                }
            }
            '*' => Matched(self.make_token(Star)),
            '!' => next_is_or!('=', BangEqual, Bang, self),
            '=' => next_is_or!('=', EqualEqual, Equal, self),
            '<' => next_is_or!('=', LessEqual, Less, self),
            '>' => next_is_or!('=', GreaterEqual, Greater, self),
            // '"' => MatchState::StringLit,
            '"' => {
                while !self.next_is('"') {
                    match self.move_to_next_char() {
                        None => return SyntaxError,
                        Some('\n') => {
                            self.line += 1;
                        }
                        _ => {}
                    }
                }
                self.move_to_next_char();
                Matched(self.make_token(String))
            }
            '\n' => ScanNextLine,
            ' ' | '\t' => ScanNext,
            _ => SyntaxError,
        }
    }

    fn move_to_next_char(&mut self) -> Option<char> {
        let next_ = self.chars.next();
        if next_.is_some() {
            self.cur_pos += 1;
        }
        next_
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
}
