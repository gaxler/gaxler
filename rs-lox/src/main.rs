mod errors;
mod opcode;
mod parser;
mod scanner;
mod vm;
mod value;
mod session;

use std::cell::RefCell;
use std::fs;
use std::mem::size_of;
use session::RuntimeContext;
use vm::disassemble_op;
use vm::VM;
// use scanner::dummy_compile;
use parser::compile;
use std::env;
use opcode::Chunk;

use crate::value::Value;

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("=== {} ===", name);

    for (idx, _) in chunk.ops.iter().enumerate() {
        disassemble_op(chunk, idx);
    }
}

fn shitcode() {
    use opcode::*;

    println!("Size of OpCode is {} bytes", size_of::<OpCode>());
    println!("Size of Value is {} bytes", size_of::<Value>());
    println!("Size of String is {} bytes", size_of::<String>());
    println!("Size of Vec is {} bytes", size_of::<Vec<String>>());
    println!("Size of Vec is {} bytes", size_of::<Vec<*mut u8>>());


}

fn interpret(source: &str) {
    let chunk = compile(source);
    let mut vm = VM::init(true);
    vm.load_chunk(chunk);
    vm.run().unwrap();
    let res = vm.pop();
    println!("[Res]: {}", res);
}


fn repl_callback(input: &str) -> Vec<String> {
    let ret: Vec<&str>;
    if input.starts_with("g") {
        ret = vec!["grisha", "gregory"];
    } else {
        ret = vec!["einav"];
    }

    ret.iter().map(|s| s.to_string()).collect()
}

fn repl() {
    // linenoise::set_callback(repl_callback);
    linenoise::set_multiline(3);

    let mut runtime = RuntimeContext::start(true);


    loop {
        let val = linenoise::input("> ");
        if let Some(cmd) = val {
            match cmd.as_str() {
                ":q" => {
                    runtime.debug_report();
                    break
                },
                // s => interpret(s),
                s => { 
                    let expr_id = runtime.compile(s); 
                    runtime.exec(expr_id);

                }
            }
        }
    }
}

fn main() {

    match env::args().nth(1) {
        None => repl(),
        Some(txt) => {
            if txt == "info" {
                shitcode();
                return ;
            }
            let source = fs::read_to_string(txt).unwrap();
            interpret(&source);
        }
    }
}
