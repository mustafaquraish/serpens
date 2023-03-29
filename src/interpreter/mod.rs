use crate::ast::AST;
use crate::common::{get, make, Ref, Span};
use crate::error::{runtime_error as error, Result};
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::value::{IteratorValue, Value};

mod builtin;
pub mod value;

#[derive(Debug)]
pub struct Scope {
    pub vars: HashMap<String, Ref<Value>>,
    pub parent: Option<Ref<Scope>>,
    pub in_function: bool,
}

impl Scope {
    pub fn new(parent: Option<Ref<Scope>>, in_function: bool) -> Ref<Scope> {
        make!(Scope {
            vars: HashMap::new(),
            parent,
            in_function,
        })
    }

    fn insert(&mut self, name: &str, value: Ref<Value>, update: bool, loc: &Span) -> Result<()> {
        if !update || self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
        } else {
            match &self.parent {
                Some(parent) => parent.borrow_mut().insert(name, value, update, loc)?,
                None => error!(loc, "Variable {} not found, couldn't update", name),
            }
        }
        Ok(())
    }

    fn get(&self, name: &str) -> Option<Ref<Value>> {
        if self.vars.contains_key(name) {
            self.vars.get(name).cloned()
        } else {
            match &self.parent {
                Some(parent) => parent.borrow_mut().get(name),
                None => None,
            }
        }
    }
}

#[derive(Debug)]
enum ControlFlow {
    None,
    Continue,
    Break,
    Return(Ref<Value>),
}

type BuiltInFunctionType = fn(&Span, Vec<Ref<Value>>) -> Result<Ref<Value>>;

pub struct Interpreter {
    builtins: HashMap<&'static str, BuiltInFunctionType>,
    control_flow: ControlFlow,
    the_nothing: Ref<Value>,
}

macro_rules! builtins {
    ($($name:ident),+ $(,)?) => {
        HashMap::from([$(
            (
                stringify!($name),
                builtin::$name as BuiltInFunctionType,
            ),
        )+])
    };
}

impl Interpreter {
    pub fn new() -> Self {
        let builtins = builtins!(print, len, exit);
        Self {
            builtins,
            control_flow: ControlFlow::None,
            the_nothing: make!(Value::Nothing),
        }
    }

    pub fn execute(&mut self, ast: &Rc<AST>) -> Result<Ref<Value>> {
        let scope = Scope::new(None, false);
        self.run(ast, scope)
    }

    pub fn run_block_without_new_scope(
        &mut self,
        ast: &Rc<AST>,
        scope: Ref<Scope>,
    ) -> Result<Ref<Value>> {
        match ast.as_ref() {
            AST::Block(_, stmts) => {
                let mut last = None;
                for stmt in stmts {
                    last = Some(self.run(stmt, scope.clone())?);
                }
                Ok(last.unwrap_or_else(|| self.the_nothing.clone()))
            }
            _ => unreachable!("run_block_without_scope called on non-block"),
        }
    }

    fn run(&mut self, ast: &Rc<AST>, scope: Ref<Scope>) -> Result<Ref<Value>> {
        macro_rules! dispatch_op {
            ($span:expr, $op:path, $left:expr, $right:expr) => {{
                let left = self.run($left, scope.clone())?;
                let right = self.run($right, scope.clone())?;
                $op(left, right, $span)?
            }};

            ($span:expr, $op:path, $val:expr) => {{
                let val = self.run($val, scope.clone())?;
                $op(val, $span)?
            }};
        }
        Ok(match ast.as_ref() {
            // Literals
            AST::BooleanLiteral(_, value) => make!(Value::Boolean(*value)),
            AST::IntegerLiteral(_, num) => make!(Value::Integer(*num)),
            AST::FloatLiteral(_, num) => make!(Value::Float(*num)),
            AST::StringLiteral(_, string) => make!(Value::String(string.clone())),
            AST::Nothing(_) => self.the_nothing.clone(),

            AST::Plus(span, left, right) => dispatch_op!(span, Value::plus, left, right),
            AST::Minus(span, left, right) => dispatch_op!(span, Value::minus, left, right),
            AST::Multiply(loc, left, right) => dispatch_op!(loc, Value::multiply, left, right),
            AST::Divide(loc, left, right) => dispatch_op!(loc, Value::divide, left, right),

            AST::Not(loc, expr) => dispatch_op!(loc, Value::not, expr),
            AST::And(loc, left, right) => dispatch_op!(loc, Value::and, left, right),
            AST::Or(loc, left, right) => dispatch_op!(loc, Value::or, left, right),

            AST::Equals(loc, left, right) => dispatch_op!(loc, Value::equals, left, right),
            AST::NotEquals(loc, left, right) => dispatch_op!(loc, Value::not_equals, left, right),
            AST::LessThan(loc, left, right) => dispatch_op!(loc, Value::less_than, left, right),

            AST::GreaterThan(loc, left, right) => {
                dispatch_op!(loc, Value::greater_than, left, right)
            }
            AST::LessEquals(loc, left, right) => {
                dispatch_op!(loc, Value::less_equals, left, right)
            }
            AST::GreaterEquals(loc, left, right) => {
                dispatch_op!(loc, Value::greater_equals, left, right)
            }

            AST::Call(span, func, args) => self.handle_call(scope, span, func, args)?,

            AST::Function {
                name,
                args,
                body,
                span,
                ..
            } => {
                let func = make!(Value::Function {
                    span: *span,
                    name: name.clone().unwrap_or_else(|| "<anon>".to_string()),
                    args: args.clone(),
                    body: body.clone(),
                    scope: scope.clone(),
                });
                match name {
                    Some(name) => scope
                        .borrow_mut()
                        .insert(name, func.clone(), false, span)?,
                    None => {}
                }
                func
            }

            AST::Slice {
                span,
                lhs,
                start,
                end,
                step,
            } => {
                let lhs = self.run(lhs, scope.clone())?;
                let start = start
                    .clone()
                    .map(|start| self.run(&start, scope.clone()))
                    .transpose()?;
                let end = end
                    .clone()
                    .map(|end| self.run(&end, scope.clone()))
                    .transpose()?;
                let step = step
                    .clone()
                    .map(|step| self.run(&step, scope.clone()))
                    .transpose()?;
                Value::slice(lhs, start, end, step, span)?
            }

            AST::Block(..) => {
                let block_scope = Scope::new(Some(scope.clone()), scope.borrow().in_function);
                self.run_block_without_new_scope(ast, block_scope)?
            }

            AST::Variable(span, name) => {
                if self.builtins.get(name.as_str()).is_some() {
                    make!(Value::BuiltInFunction(name.clone()))
                } else if let Some(value) = scope.borrow_mut().get(name) {
                    value
                } else {
                    error!(span, "Variable {} not found", name)
                }
            }

            AST::Return(span, val) => {
                if !scope.borrow_mut().in_function {
                    error!(span, "Return statement outside of function")
                }
                self.control_flow = ControlFlow::Return(self.run(val, scope)?);
                self.the_nothing.clone()
            }

            AST::Assignment(span, lhs, value) => {
                let value = self.run(value, scope.clone())?;
                match lhs.as_ref() {
                    AST::Variable(span, name) => {
                        if scope.borrow_mut().get(name).is_none() {
                            error!(span, "Variable {} doesn't exist", name)
                        }
                        if self.builtins.contains_key(name.as_str()) {
                            error!(span, "`{}` is a built-in function, can't override it", name)
                        }
                        scope
                            .borrow_mut()
                            .insert(name, value.clone(), true, span)?;
                        value
                    }
                    _ => error!(span, "Can't assign to {:?}", lhs),
                }
            }

            AST::VarDeclaration(span, name, value) => {
                if self.builtins.contains_key(name.as_str()) {
                    error!(
                        span,
                        "`{}` is a built-in function, can't be used as a variable", name
                    )
                }
                let value = self.run(value, scope.clone())?;
                scope
                    .borrow_mut()
                    .insert(name, value.clone(), false, span)?;
                value
            }

            AST::Assert(loc, cond) => {
                let cond = self.run(cond, scope)?;
                match get!(cond) {
                    Value::Boolean(true) => {}
                    Value::Boolean(false) => error!(loc, "Assertion failed"),
                    _ => error!(loc, "Assertion condition must be a boolean"),
                }
                self.the_nothing.clone()
            }

            AST::If(span, cond, body, else_body) => {
                let cond = self.run(cond, scope.clone())?;
                #[allow(clippy::let_and_return)]
                let res = match get!(cond) {
                    Value::Boolean(true) => self.run(body, scope)?,
                    Value::Boolean(false) => match else_body {
                        Some(else_body) => self.run(else_body, scope)?,
                        None => self.the_nothing.clone(),
                    },
                    _ => error!(span, "If condition must be a boolean"),
                };
                res
            }

            AST::While(span, cond, body) => {
                loop {
                    let cond = self.run(cond, scope.clone())?;
                    match get!(cond) {
                        Value::Boolean(true) => {
                            self.run(body, scope.clone())?;
                            match self.control_flow {
                                ControlFlow::None => {}
                                ControlFlow::Continue => self.control_flow = ControlFlow::None,
                                ControlFlow::Break => {
                                    self.control_flow = ControlFlow::None;
                                    break;
                                }
                                ControlFlow::Return(_) => break,
                            }
                        }
                        Value::Boolean(false) => break,
                        _ => error!(span, "While condition must be a boolean"),
                    };
                }
                self.the_nothing.clone()
            }

            AST::For(span, loop_var, iter, body) => {
                let val = self.run(iter, scope.clone())?;
                let iter = Value::iterator(val, span)?;
                match get!(iter) {
                    Value::Iterator(IteratorValue(iter)) => {
                        let iter = &mut *(*iter).borrow_mut();
                        for val in iter {
                            let loop_scope =
                                Scope::new(Some(scope.clone()), scope.borrow_mut().in_function);
                            loop_scope
                                .borrow_mut()
                                .insert(loop_var, val.clone(), false, span)?;
                            self.run(body, loop_scope)?;
                            match self.control_flow {
                                ControlFlow::None => {}
                                ControlFlow::Continue => self.control_flow = ControlFlow::None,
                                ControlFlow::Break => {
                                    self.control_flow = ControlFlow::None;
                                    break;
                                }
                                ControlFlow::Return(_) => break,
                            }
                        }
                    }
                    _ => error!(span, "For loop must iterate over an iterable"),
                };
                self.the_nothing.clone()
            }

            AST::Range(span, start, end) => {
                let start = self.run(start, scope.clone())?;
                let end = self.run(end, scope)?;
                Value::create_range(start, end, span)?
            }

            AST::Break(_) => {
                self.control_flow = ControlFlow::Break;
                self.the_nothing.clone()
            }
            AST::Continue(_) => {
                self.control_flow = ControlFlow::Continue;
                self.the_nothing.clone()
            }

            AST::Index(span, left, right) => {
                let left = self.run(left, scope.clone())?;
                let right = self.run(right, scope)?;
                Value::index(left, right, span)?
            }
        })
    }

    fn handle_call(
        &mut self,
        scope: Ref<Scope>,
        span: &Span,
        func: &Rc<AST>,
        args: &[Rc<AST>],
    ) -> Result<Ref<Value>> {
        let func = self.run(func, scope.clone())?;
        let args = args
            .iter()
            .map(|arg| self.run(arg, scope.clone()))
            .collect::<Result<Vec<_>>>()?;

        return Ok(match get!(func) {
            Value::Function {
                body,
                args: func_args,
                scope: closure_scope,
                ..
            } => {
                let new_scope = Scope::new(Some(closure_scope.clone()), true);
                if args.len() != func_args.len() {
                    error!(
                        *span,
                        "Expected {} arguments, got {}",
                        func_args.len(),
                        args.len()
                    )
                }
                for (arg, value) in func_args.iter().zip(args) {
                    new_scope.borrow_mut().insert(arg, value, false, span)?;
                }
                self.run(body, new_scope)?;
                let value = if let ControlFlow::Return(value) = &self.control_flow {
                    value.clone()
                } else {
                    self.the_nothing.clone()
                };
                self.control_flow = ControlFlow::None;
                value
            }
            Value::BuiltInFunction(func) => match self.builtins.get(func.as_str()) {
                Some(func) => func(span, args)?,
                None => error!(span, "Built-in function {} not found", func),
            },
            x => error!(span, "Can't call object {:?}", x),
        });
    }
}
