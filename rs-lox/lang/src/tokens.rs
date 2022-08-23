use crate::scanner::Scanner;

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
    pub fn make(token_type: TokenType, scanner: &Scanner) -> Self {
        let (start_pos, len) = match token_type {
            TokenType::String => (
                scanner.start_pos + 1,
                scanner.cur_pos - scanner.start_pos - 2,
            ),
            _ => (scanner.start_pos, scanner.cur_pos - scanner.start_pos),
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

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Precedence {
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
