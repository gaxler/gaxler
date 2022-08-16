use std::{iter, borrow::Borrow};

type CountTy = i16;

#[derive(Debug, Clone)]
pub struct Local {
    name: String,
    depth: CountTy,
}

impl Default for Local {
    fn default() -> Self {
        Self { name: "".to_string(), depth: -1 }
    }
}

const LOCAL_MAX: usize = u8::MAX as usize + 1;

pub struct Compiler {
    locals: Vec<Local>,
    count: CountTy,
    depth: CountTy,
}

impl Compiler {
    pub fn init() -> Self {
        Self {
            count: 0,
            depth: 0,
            locals: vec![Default::default(); LOCAL_MAX],
        }
    }

    #[inline]
    pub fn begin_scope(&mut self) {
        self.depth += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.depth -= 1;
    }

    #[inline]
    pub fn local_scope(&self) -> bool {
        self.depth > 0
    }

    pub fn add_local(&mut self, ident_: String) {
        let local = Local {
            name: ident_,
            depth: self.depth,
        };

        self.locals[self.count as usize] = local;
        self.count += 1;
    }

    /// See if variable with same name exeists in current scope.
    /// We only need to check the current scope since we pop everything out when levaing scopes
    /// Upper scopes are of no interest to us since it's OK to shadow them in lower scopes
    pub fn local_exists(&self, name: &str) -> Option<()> {
        let res = self.locals[..(self.count as usize)]
            .iter()
            .cloned()
            .enumerate()
            .rev()
            .take_while(|(_, l)| (l.depth >= self.depth) && l.depth > -1)
            .fold(false, |acc, (_, l)| l.name == name || acc);
        // for now i want to pattern match this
        if res {
            dbg!(name);
            dbg!(self.locals[..(self.count as usize)].to_vec());
            None
        } else {
            Some(())
        }
    }

    pub fn find_local(&self, name: &str) -> Option<u8> {
        for (slot, l) in self.locals[..(self.count as usize)]
            .iter()
            .enumerate()
            .rev()
        {
            if l.name == name {
                return Some(slot as u8);
            }
        }
        None
    }
    /// Tells you if top of the locals is in current scope
    /// used to
    pub fn should_pop_local(&mut self) -> bool {
        if self.count <= 0 {
            return false;
        }

        let idx = (self.count - 1) as usize;
        let l = self.locals[idx].borrow();
        if self.depth == l.depth {
            self.count -= 1;
            return true;
        }
        false
    }
}

impl<'a> std::fmt::Display for Compiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Locals: {} Cur Depth: {}>", self.count, self.depth)
    }
}
