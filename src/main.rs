use crate::error::Result;

mod ast;
mod builtin;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod common;
// mod repl;
mod token;
mod value;

fn main() -> Result<()> {
    let filename = "bench.sp";
    let content = std::fs::read_to_string(filename).expect("Couldn't open input file");
    let mut lex = lexer::Lexer::new(content.to_string(), filename.to_string());
    let tokens = lex.lex()?;
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;
    let mut interpreter = interpreter::Interpreter::new();
    interpreter.execute(&ast)?;
    Ok(())
}
