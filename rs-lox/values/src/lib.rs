mod chunk;
mod stack;
mod value;
mod var_store;

pub use value::Value;

pub use chunk::Chunk;
pub use stack::Stack;
pub use var_store::VarStore;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
