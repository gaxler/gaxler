mod errors;
mod opcode;
mod parser;
mod scanner;
mod vm;
mod value;
mod session;

use std::fs;
use std::mem::{size_of, align_of};
use session::RuntimeContext;
use std::env;

use crate::value::Value;


fn shitcode() {
    use opcode::*;

    println!("OpCode: size is {} bytes", size_of::<OpCode>());
    println!("Value: size is {} bytes, align is {} bytes", size_of::<Value>(), align_of::<Value>());
    println!("String: size {} bytes, align {} bytes ", size_of::<String>(), align_of::<String>());
    println!("Size of String Vec is {} bytes", size_of::<Vec<String>>());
    println!("Size of Pointer Vec is {} bytes", size_of::<Vec<*mut u8>>());

}

fn interpret(source: &str, debug: bool) {

    let mut runtime = RuntimeContext::start(debug);
    let ch_id = match runtime.compile(source) {
        Ok(idx) => idx,
        Err(e) => {
            println!(" Error: [{}]", e);
            return;
        }
        
    };
    match runtime.exec(ch_id) {
        Ok(_) => {},
        Err(e) => {
            println!(" Error: [{}]", e);
            return; 
        }
    }
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

    let mut runtime = RuntimeContext::start(false);


    loop {
        let val = linenoise::input("> ");
        if let Some(cmd) = val {
            match cmd.as_str() {
                ":q" => {
                    runtime.debug_report();
                    break
                },
                ":verbose" => {
                    
                }
                // s => interpret(s),
                s => { 
                    let expr_id = runtime.compile(s); 
                    match expr_id {
                        Err(e) => println!("  Err: [{}]", e),
                        Ok(idx) => {
                            if let Err(e) = runtime.exec(idx) {
                                println!("  Err: [{}]", e); 
                            }
                        },
                    }
                }
            }
        }
    }
}

fn main() {
    match (env::args().nth(1), env::args().nth(2)) {
        (None, _) => repl(),
        (Some(txt), debug) => {
            if txt == "info" {
                shitcode();
                return ;
            }
            let dbg = debug.is_some();
            let source = fs::read_to_string(txt).unwrap();
            interpret(&source, dbg);
        }
    }
}
