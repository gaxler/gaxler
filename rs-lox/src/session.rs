
use crate::{vm::VM, parser::Parser, errors::{COMPError, RTError}};
use lang::Scanner;
use values::Chunk;

pub type ChunkAddr = usize;

pub struct RuntimeContext {
    vm: VM,
    chunks: Vec<Option<Chunk>>
}

impl RuntimeContext {

    pub fn start(debug: bool) -> Self {
        // let heap = RefCell::new(Heap::init());
        let vm = VM::init(debug);
        Self {vm, chunks: vec![]}
    }

    pub fn compile(&mut self, source: &str) -> COMPError<ChunkAddr> {
        let mut scanner = Scanner::from_str(source).unwrap();
        let mut chunk = Chunk::new();

        // let mut parser = Parser::init(&mut scanner, &mut chunk, self.heap.borrow_mut());
        let mut parser = Parser::init(&mut scanner, &mut chunk);
        parser.parse()?;

        self.chunks.push(Some(chunk));
        Ok(self.chunks.len() - 1)
    }

    pub fn get_chunk(&mut self, addr: ChunkAddr) -> Chunk {
        self.chunks[addr].take().unwrap()
    }

    pub fn put_chunk(&mut self, addr: ChunkAddr, chunk: Chunk) {
        self.chunks[addr] = Some(chunk);
    }

    pub fn exec(&mut self, addr: ChunkAddr) -> RTError<()> {
        let cur_chunk = self.get_chunk(addr);
        self.vm.load_chunk(cur_chunk);
        let res = self.vm.run();
        // make sure to put the chunk back even if the execution fails
        let cur_chunk = self.vm.unload_chunk();
        self.put_chunk(addr, cur_chunk);
        res
    }

    pub fn debug_report(&self) {
        self.vm.show_stack();
    }






}
