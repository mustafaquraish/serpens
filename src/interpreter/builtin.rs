use crate::error::{runtime_error as error, Result};
use crate::interpreter::value::Value;
use crate::common::{get, make, Ref, Span};

pub fn print(_span: &Span, args: Vec<Ref<Value>>) -> Result<Ref<Value>> {
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            print!(" ");
        }
        match get!(arg) {
            Value::Integer(num) => print!("{}", num),
            Value::Float(num) => print!("{}", num),
            Value::String(string) => print!("{}", string),
            Value::Boolean(boolean) => print!("{}", boolean),
            Value::Nothing => print!("nothing"),
            Value::Iterator(_) => print!("<iterator>"),
            Value::Range(start, end) => print!("{}..{}", start, end),
            arg => print!("{:?}", arg),
        }
    }
    println!();
    Ok(make!(Value::Nothing))
}

pub fn len(span: &Span, args: Vec<Ref<Value>>) -> Result<Ref<Value>> {
    if args.len() != 1 {
        error!(span, "len() takes exactly one argument");
    }

    Ok(match get!(&args[0]) {
        Value::String(string) => make!(Value::Integer(string.len() as i64)),
        other => error!(span, "len() does not support {:?}", other),
    })
}

pub fn exit(span: &Span, args: Vec<Ref<Value>>) -> Result<Ref<Value>> {
    let code = match args.get(0) {
        Some(val) => match get!(val) {
            Value::Integer(i) => *i,
            _ => error!(span, "exit() may only take an integer as argument"),
        }
        None => 0,
    };

    match code.try_into() {
        Ok(code) => std::process::exit(code),
        Err(_) => std::process::exit(1),
    }
}
