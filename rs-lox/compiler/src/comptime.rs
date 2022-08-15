use lang::Token;

type CountTy = u16;

#[derive(Debug, Clone, Copy)]
pub struct Local {
    name: Token,
    depth: CountTy
}

const LOCAL_MAX: usize = u8::MAX as usize + 1;

pub struct Compiler {
    locals: [Local; LOCAL_MAX],
    count: CountTy,
    depth: CountTy
}

impl Compiler {
    pub fn init() -> Self{
        Self {
            count:0,
            depth: 0,
            locals: [Local {name: Token::empty(0), depth: 0}; LOCAL_MAX]
        }
    }

    #[inline]
    pub fn begin_scope(&mut self) {
        self.depth += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.depth -=1;
    }
}

impl std::fmt::Display for Compiler  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Locals: {} Cur Depth: {}>", self.count, self.depth)
    }
}