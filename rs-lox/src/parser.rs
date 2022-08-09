use crate::scanner::Scanner;
use crate::scanner::Token;

use crate::opcode::Chunk;

pub struct Parser {
    cur: Token,
    prev: Token,
    had_err: bool,
    panic_mode: bool
}

