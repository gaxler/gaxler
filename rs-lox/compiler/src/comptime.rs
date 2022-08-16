type CountTy = i16;

#[derive(Debug, Clone, Copy)]
pub struct Local<'a> {
    name: &'a str,
    depth: CountTy,
}

const LOCAL_MAX: usize = u8::MAX as usize + 1;

pub struct Compiler<'a> {
    locals: [Local<'a>; LOCAL_MAX],
    count: CountTy,
    depth: CountTy,
}

impl<'a> Compiler<'a> {
    pub fn init() -> Self {
        Self {
            count: 0,
            depth: 0,
            locals: [Local {
                name: "",
                depth: -1,
            }; LOCAL_MAX],
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

    pub fn add_local(&mut self, ident_: &'a str) {
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
            .enumerate()
            .rev()
            .take_while(|(_, &l)| (l.depth >= self.depth) && l.depth > -1)
            .fold(false, |acc, (_, &l)| l.name == name || acc);
        // for now i want to pattern match this
        if res {
            dbg!(name);
            dbg!(self.locals[..(self.count as usize)].to_vec());
            None
        } else {
            Some(())
        }

        // .fold(None, |acc, (idx, &l)| match acc {
        // Some(prev) => Some(prev),
        // None => {
        //     if l.name == name {
        //         Some(idx)
        //     } else {
        //         None
        //     }
        // }
        // })
    }

    /// Tells you if top of the locals is in current scope
    /// used to 
    pub fn should_pop_local(&mut self) -> bool {
        if self.count <= 0 {
            return false;
        }
        
        let idx = (self.count - 1) as usize;
        let l = self.locals[idx];
        if self.depth == l.depth {
            self.count -= 1;
            return true;
        }
        false
    }
}

impl<'a> std::fmt::Display for Compiler<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Locals: {} Cur Depth: {}>", self.count, self.depth)
    }
}
