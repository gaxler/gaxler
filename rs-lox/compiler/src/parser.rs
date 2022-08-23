// use std::hash::BuildHasherDefault;

// use values::Value;

// use crate::{Compiler, Local};

// use lang::{ConstIdx, InstructAddr, OpCode};
// use lang::{Precedence, Scanner, Token, TokenType};
// use values::Chunk;


// #[repr(u8)]
// enum State {
//     Init,
//     VarDec,
// }

// impl Default for State {
//     fn default() -> Self {
//         Self::Init
//     }
// }

// pub struct Parser<'a> {
//     scanner: Scanner<'a>,
//     chunk: Chunk,
//     compiler: Compiler,
//     state: Vec<State>
// }

// impl<'a> Parser<'a> {
//     pub fn parse(scanner: Scanner<'a>) -> Chunk {
//         let mut chunk = Chunk::new();
//         let mut compiler = Compiler::init();
//         let state = Self {scanner, chunk, compiler, state: vec![State::Init]};
//         state.get_chunk()
//     }

//     fn get_chunk(&mut self) -> Chunk {
        
//         'token: loop {
//             let tok = self.next();
//             'state: loop {
//                 let state = self.state.last();
            
//                 if state.is_none() {
//                     // we have an empty state this means we are done
//                     // or maybe something is wrong, not sure yet
//                     todo!()
//                 }
                
//                 match (state.unwrap(), tok.ty) {
//                     (State::Init, TokenType::Var) => {
//                         self.set(State::VarDec);
//                         continue 'token;
//                     },
//                     (State::VarDec, TokenType::Ident) => 
//                     _ => todo!()
//                 }


//         }}

//         self.chunk
//     }

//     fn set(&mut self, state:State) {
//         self.state.push(state);
//     }

//     fn next(&mut self) -> Token {
//         match self.scanner.scan_token() {
//             Ok(tok) => tok,
//             Err(e) => {
//                 dbg!(e);
//                 panic!("Scanner Error")
//             }
            
//         }
//     } 
    
// }