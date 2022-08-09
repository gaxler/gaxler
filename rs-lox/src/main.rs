mod scanner;
mod opcode;
mod vm;
mod parser;

use opcode::*;
use std::fs;
use std::mem::size_of;
use vm::disassemble_op;
use vm::VM;
use scanner::{dummy_compile, compile};


enum RunMode {
    Repl,
    File,
    ShitCode,
}

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("=== {} ===", name);

    for (idx, _) in chunk.ops.iter().enumerate() {
        disassemble_op(chunk, idx);
    }
}

fn shitcode() {
    println!("Size of OpCode is {} bytes", size_of::<OpCode>());
    let mut chunk = Chunk::new();
    chunk.add_const(Value::Int(28));
    chunk.add_op(OpCode::CONSTANT(0), 1);
    chunk.add_op(OpCode::NEGATE, 2);
    chunk.add_op(OpCode::ADD, 3);
    chunk.add_op(OpCode::RETURN, 3);
    let mut vm = VM::init(&chunk, true);
    vm.push(Value::Float(9.14));
    vm.push(Value::Float(9.14));

    println!("\n=== Execution ===");

    vm.run().unwrap();
}

fn interpret(source: &str) {
    // let mut chunk = compile(source);
    // let mut vm = VM::init(&chunk, true);
    // vm.run();

    // dummy_compile(source).unwrap();
   let chunk = compile(source);
   let mut vm = VM::init(&chunk, true);
   vm.run().unwrap();
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

    loop {
        let val = linenoise::input("> ");
        if let Some(cmd) = val {
            match cmd.as_str() {
                ":q" => break,
                s => interpret(s),
            }
        }
    }
}

fn main() {
    let mode = RunMode::File;

    match mode {
        RunMode::File => {
            let source = fs::read_to_string("expr.lox").unwrap();
            interpret(&source);

        }
        RunMode::Repl => {
            repl();
        }

        RunMode::ShitCode => {
            shitcode();
        }
    }
}
