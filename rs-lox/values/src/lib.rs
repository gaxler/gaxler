mod value;
mod chunk;
mod var_store;
mod stack;

pub use value::Value;

pub use chunk::Chunk;
pub use var_store::VarStore;
pub use stack::Stack;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
