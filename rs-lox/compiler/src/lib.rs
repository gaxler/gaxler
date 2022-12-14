mod comptime;
mod parser;

pub use comptime::{Compiler, Local};
pub use parser::COMPError;
pub use parser::Parser;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
