use crate::ast::AST;
use crate::common::{Ref, get, make};
use crate::token::Span;
use crate::error::{Result, runtime_error as error};
use crate::interpreter::Scope;
use std::rc::Rc;

pub struct IteratorValue(pub Ref<dyn Iterator<Item = Ref<Value>>>);

struct StringIterator {
    string: String,
    index: usize,
}

impl Iterator for StringIterator {
    type Item = Ref<Value>;

    fn next(&mut self) -> Option<Ref<Value>> {
        if self.index >= self.string.len() {
            None
        } else {
            let c = self.string.chars().nth(self.index).unwrap();
            self.index += 1;
            Some(make!(Value::String(c.to_string())))
        }
    }
}

impl IteratorValue {
    pub fn for_string(string: &String) -> IteratorValue {
        IteratorValue(make!(StringIterator { string: string.clone(), index: 0 }))
    }

    pub fn for_range(start: &i64, end: &i64) -> IteratorValue {
        IteratorValue(make!((*start..*end).map(|v| make!(Value::Integer(v)))))
    }
}

impl std::fmt::Debug for IteratorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<iterator>")
    }
}

#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    BuiltInFunction(String),
    Iterator(IteratorValue),
    Function {
        span: Span,
        name: String,
        body: Rc<AST>,
        args: Vec<String>,
        scope: Ref<Scope>,
    },
    Range(i64, i64),
    Nothing,
}

impl Value {
    pub fn plus(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Integer(left), Value::Integer(right)) => make!(Value::Integer(*left + *right)),
            (Value::Integer(left), Value::Float(right)) => make!(Value::Float(*left as f64 + *right)),
            (Value::Float(left), Value::Float(right)) => make!(Value::Float(*left + *right)),
            (Value::Float(left), Value::Integer(right)) => make!(Value::Float(*left + *right as f64)),
            (Value::String(left), Value::String(right)) => make!(Value::String(left.clone() + right)),
            _ => error!(span, "Invalid types for addition"),
        })
    }

    pub fn minus(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Integer(left), Value::Integer(right)) => make!(Value::Integer(*left - *right)),
            (Value::Integer(left), Value::Float(right)) => make!(Value::Float(*left as f64 - *right)),
            (Value::Float(left), Value::Float(right)) => make!(Value::Float(*left - *right)),
            (Value::Float(left), Value::Integer(right)) => make!(Value::Float(*left - *right as f64)),
            _ => error!(span, "Invalid types for subtraction"),
        })
    }

    pub fn multiply(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Integer(left), Value::Integer(right)) => make!(Value::Integer(*left * *right)),
            (Value::Integer(left), Value::Float(right)) => make!(Value::Float(*left as f64 * *right)),
            (Value::Float(left), Value::Float(right)) => make!(Value::Float(*left * *right)),
            (Value::Float(left), Value::Integer(right)) => make!(Value::Float(*left * *right as f64)),
            (Value::String(left), Value::Integer(right)) => {
                if *right < 0 {
                    error!(span, "{right} is not a positive integer.")
                }
                make!(Value::String(left.repeat(*right as usize)))
            }
            _ => error!(span, "Invalid types for multiplication"),
        })
    }

    pub fn divide(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Integer(left), Value::Integer(right)) => make!(Value::Integer(*left / *right)),
            (Value::Integer(left), Value::Float(right)) => make!(Value::Float(*left as f64 / *right)),
            (Value::Float(left), Value::Float(right)) => make!(Value::Float(*left / *right)),
            (Value::Float(left), Value::Integer(right)) => make!(Value::Float(*left / *right as f64)),
            _ => error!(span, "Invalid types for division"),
        })
    }

    pub fn slice(
        lhs: Ref<Value>,
        start: Option<Ref<Value>>,
        end: Option<Ref<Value>>,
        step: Option<Ref<Value>>,
        span: &Span,
    ) -> Result<Ref<Value>> {

        let start = start.unwrap_or(make!(Value::Integer(0)));
        let step = step.unwrap_or(make!(Value::Integer(1)));
        match get!(lhs) {
            Value::String(s) => {
                let end = end.unwrap_or(make!(Value::Integer(s.len() as i64)));
                match (get!(start), get!(end), get!(step)) {
                    (Value::Integer(start), Value::Integer(end), Value::Integer(step)) => {
                        if *step == 0 {
                            error!(span, "Step cannot be 0")
                        }
                        let mut result = String::new();
                        let mut i = *start;
                        while i < *end {
                            result.push(s.chars().nth(i as usize).unwrap());
                            i += *step;
                        }
                        return Ok(make!(Value::String(result)))
                    }
                    _ => error!(span, "Invalid types for slice"),
                };
            },
            _ => error!(span, "Can only slice strings"),
        }
    }

    pub fn not(val: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match get!(val) {
            Value::Boolean(b) => make!(Value::Boolean(!b)),
            _ => error!(span, "Invalid type for not"),
        })
    }
    pub fn and(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Boolean(left), Value::Boolean(right)) => make!(Value::Boolean(*left && *right)),
            _ => error!(span, "Invalid types for and"),
        })
    }
    pub fn or(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Boolean(left), Value::Boolean(right)) => make!(Value::Boolean(*left || *right)),
            _ => error!(span, "Invalid types for or"),
        })
    }

    pub fn equals(left: Ref<Value>, right: Ref<Value>, _span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Integer(left), Value::Integer(right)) => make!(Value::Boolean(*left == *right)),
            (Value::Integer(left), Value::Float(right)) => make!(Value::Boolean(*left as f64 == *right)),
            (Value::Float(left), Value::Float(right)) => make!(Value::Boolean(*left == *right)),
            (Value::Float(left), Value::Integer(right)) => make!(Value::Boolean(*left == *right as f64)),
            (Value::String(left), Value::String(right)) => make!(Value::Boolean(*left == *right)),
            (Value::Boolean(left), Value::Boolean(right)) => make!(Value::Boolean(*left == *right)),
            _ => make!(Value::Boolean(false)),
        })
    }
    pub fn not_equals(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Value::not(Value::equals(left, right, span)?, span)
    }
    pub fn less_than(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Integer(left), Value::Integer(right)) => make!(Value::Boolean(*left < *right)),
            (Value::Integer(left), Value::Float(right)) => make!(Value::Boolean((*left as f64) < *right)),
            (Value::Float(left), Value::Float(right)) => make!(Value::Boolean(*left < *right)),
            (Value::Float(left), Value::Integer(right)) => make!(Value::Boolean(*left < *right as f64)),
            (Value::String(left), Value::String(right)) => make!(Value::Boolean(*left < *right)),
            _ => error!(span, "Invalid types for less than"),
        })
    }

    pub fn greater_than(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Value::less_than(right, left, span)
    }

    pub fn less_equals(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(left), get!(right)) {
            (Value::Integer(left), Value::Integer(right)) => make!(Value::Boolean(*left <= *right)),
            (Value::Integer(left), Value::Float(right)) => make!(Value::Boolean((*left as f64) <= *right)),
            (Value::Float(left), Value::Float(right)) => make!(Value::Boolean(*left <= *right)),
            (Value::Float(left), Value::Integer(right)) => make!(Value::Boolean(*left <= *right as f64)),
            (Value::String(left), Value::String(right)) => make!(Value::Boolean(*left <= *right)),
            _ => error!(span, "Invalid types for less than"),
        })
    }

    pub fn greater_equals(left: Ref<Value>, right: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Value::less_equals(right, left, span)
    }

    pub fn iterator(value: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match get!(value) {
            Value::String(s) => make!(Value::Iterator(IteratorValue::for_string(s))),
            Value::Range(start, end) => make!(Value::Iterator(IteratorValue::for_range(start, end))),
            _ => error!(span, "Cannot iterate over this type"),
        })
    }

    #[allow(dead_code)]
    pub fn repr(value: Ref<Value>) -> String {
        match get!(value) {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Boolean(b) => b.to_string(),
            Value::Iterator(_) => "<iterator>".to_string(),
            Value::Function { span, name, .. } => format!("<function {}: {}>", name, span.0),
            Value::Range(start, end) => format!("{}..{}", start, end),
            Value::BuiltInFunction(name, ..) => format!("<built-in function {}>", name),
            Value::Nothing => "nothing".to_string(),
        }
    }

    pub fn create_range(start: Ref<Value>, end: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(start), get!(end)) {
            (Value::Integer(start), Value::Integer(end)) => make!(Value::Range(*start, *end)),
            _ => error!(span, "Must be integers for range"),
        })
    }

    pub fn index(value: Ref<Value>, index: Ref<Value>, span: &Span) -> Result<Ref<Value>> {
        Ok(match (get!(value), get!(index)) {
            (Value::String(value), Value::Integer(index)) => {
                match value.chars().nth(*index as usize) {
                    Some(c) => make!(Value::String(c.to_string())),
                    None => error!(span, "Index out of bounds"),
                }
            }
            _ => error!(span, "Can't index {:?} with {:?}", value, index),
        })
    }
}