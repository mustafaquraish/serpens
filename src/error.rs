use crate::token::Span;

#[derive(Debug)]
pub enum ErrorKind {
    Lexer,
    Parser,
    UnexpectedEOF,
    Runtime,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::Lexer | ErrorKind::Parser | ErrorKind::UnexpectedEOF => {
                write!(f, "SyntaxError: {}", self.message)
            }
            ErrorKind::Runtime => write!(f, "RuntimeError: {}", self.message),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! lexer_error {
    ($span:expr, $($arg:tt)*) => {
        return Err(crate::error::Error{
            kind: crate::error::ErrorKind::Lexer,
            span: $span.clone(),
            message: format!($($arg)*),
        })
    }
}
pub(crate) use lexer_error;

macro_rules! parser_error {
    ($span:expr, $($arg:tt)*) => {
        return Err(crate::error::Error{
            kind: crate::error::ErrorKind::Parser,
            span: $span.clone(),
            message: format!($($arg)*),
        })
    }
}
pub(crate) use parser_error;

macro_rules! eof_error {
    ($span:expr, $($arg:tt)*) => {
        return Err(crate::error::Error{
            kind: crate::error::ErrorKind::UnexpectedEOF,
            span: $span.clone(),
            message: format!("Unexpected EOF: {}", format!($($arg)*)),
        })
    }
}
pub(crate) use eof_error;

macro_rules! runtime_error {
    ($span:expr, $($arg:tt)*) => {
        return Err(crate::error::Error{
            kind: crate::error::ErrorKind::Runtime,
            span: $span.clone(),
            message: format!($($arg)*),
        })
    }
}
pub(crate) use runtime_error;

// TODO: refactor/remove
/*
macro_rules! _error {
    ($span:expr, $($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            let filename = &$span.filename;
            let file_content = std::fs::read_to_string(filename).expect("couldn't open input file");
            let lines = file_content.lines().collect::<Vec<&str>>();
            let context = 3;
            let min_line = if $span.line <= context {
                1
            } else {
                $span.line - context - 1
            };
            let max_line = lines.len().min($span.line + context);

            println!("╭───────────────────────────────────────────────────────────────");
            println!("│ {}: Error: {}", $span.clone(), msg);
            println!("├────┬──────────────────────────────────────────────────────────");

            for line_no in min_line..max_line {
                let line = lines[line_no];
                println!("│{:>3} │ {}", line_no, line);
                if line_no == $span.line - 1 {
                    println!("│    ├─{}┘ \x1b[0;31m{}\x1b[0m", "─".repeat($span.column - 1), msg);
                }
            }

            println!("╰────┴──────────────────────────────────────────────────────────");
            panic!();
        }
    }
}
*/
