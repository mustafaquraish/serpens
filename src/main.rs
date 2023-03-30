#![allow(clippy::upper_case_acronyms)]
// Clippy is being annoying with this one, so we'll just ignore it.
#![allow(clippy::let_and_return)]

use crate::error::Result;

mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod common;
mod repl;
mod token;
mod compiler;

fn run_file(filename: &str) -> Result<()> {
    let content = std::fs::read_to_string(filename).expect("Couldn't open input file");

    let mut lex = lexer::Lexer::new(content, Box::leak(filename.to_string().into_boxed_str()));
    let tokens = lex.lex()?;

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;

    let mut interpreter = interpreter::Interpreter::new();
    interpreter.execute(&ast)?;
    Ok(())
}

fn compile_file(filename: &str) -> Result<()> {
    let content = std::fs::read_to_string(filename).expect("Couldn't open input file");

    let mut lex = lexer::Lexer::new(content, Box::leak(filename.to_string().into_boxed_str()));
    let tokens = lex.lex()?;

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;

    let mut compiler = compiler::Compiler::new();
    let code = compiler.compile(&ast)?;

    println!("{}", code);
    Ok(())
}


fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        let mut repl = repl::Repl::new();
        repl.run();
        std::process::exit(0);
    }

    let mut filename = None;
    let mut compile = false;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-c" | "--compile" => compile = true,
            arg => {
                filename = Some(arg);
            }
        }
    }

    let filename = match filename {
        Some(filename) => filename,
        None => {
            eprintln!("No filename provided");
            std::process::exit(1);
        }
    };

    let result = if compile {
        compile_file(filename)
    } else {
        run_file(filename)
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(err) => {
            err.print_with_source();
            std::process::exit(1);
        }
    }
}
