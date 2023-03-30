use crate::ast::AST;
use crate::common::Span;
use crate::error::{compiler_error as error, Result};
// use std::collections::{HashSet};
use std::rc::Rc;


pub struct Compiler {
    buf: String,
    counter: usize,
    fn_type: String
}


impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            buf: String::new(),
            counter: 0,
            fn_type: "std::function<Ref<Value>(vector<Ref<Value>>, const char *)>".to_string(),
        }
    }

    pub fn compile(&mut self, ast: &Rc<AST>) -> Result<String> {
        self.buf.push_str("#include \"runtime/value.h\"\n\n");
        self.buf.push_str("int main() {\n");
        self.comp(ast)?;
        self.buf.push_str("}\n");
        Ok(self.buf.clone())
    }

    fn uuid(&mut self) -> String {
        let uuid = self.counter;
        self.counter += 1;
        format!("__{}", uuid)
    }

    fn comp_binary(&mut self, name: &str, left: &Rc<AST>, right: &Rc<AST>, span: &Span) -> Result<()> {
        self.comp(left)?;
        self.buf.push_str(&format!("->{}(", name));
        self.comp(right)?;
        self.buf.push_str(", ");
        self.comp_loc(span)?;
        self.buf.push_str(")");
        Ok(())
    }

    fn comp(&mut self, ast: &Rc<AST>) -> Result<()> {
        match ast.as_ref() {
            AST::IntegerLiteral(_, val) => self.buf.push_str(&format!("Value::from_int({})", val)),
            AST::StringLiteral(_, val) => self.buf.push_str(&format!("Value::from_string(\"{}\")", val)),
            AST::FloatLiteral(_, val) => self.buf.push_str(&format!("Value::from_float({})", val)),
            AST::Nothing(_) => self.buf.push_str("Nothing"),
            AST::Range(_, start, end) => self.buf.push_str(&format!("Value::from_range({}, {})", start, end)),
            AST::Plus(span, left, right) => self.comp_binary("add", left, right, span)?,
            AST::Minus(span, left, right) => self.comp_binary("sub", left, right, span)?,
            AST::Multiply(span, left, right) => self.comp_binary("mul", left, right, span)?,
            AST::Divide(span, left, right) => self.comp_binary("div", left, right, span)?,
            AST::Block(_, stmts) => {
                self.buf.push_str("{\n");
                for stmt in stmts {
                    self.comp(stmt)?;
                    self.buf.push_str(";\n");
                }
                self.buf.push_str("}");
            }
            AST::Call(span, lhs, args) => {
                match lhs.as_ref() {
                    AST::Variable(_, name) => {
                        self.comp_builtin_call(span, name, args)?;
                    }
                    _ => error!(lhs.span(), "Not implemented yet"),
                }
            }
            AST::ForEach(_, var, iter, body) => {
                let itervar = self.uuid();
                self.buf.push_str(&format!("{{ Ref<Value> {} = ", itervar));
                self.comp(iter)?;
                self.buf.push_str("->iter(");
                self.comp_loc(&iter.span())?;
                self.buf.push_str(");\n");
                self.buf.push_str(&format!("while ({}->as_iter->has_next()) {{\n", itervar));
                self.buf.push_str(&format!("  Ref<Value> {} = {}->as_iter->next();\n", var, itervar));
                self.comp(body)?;
                self.buf.push_str("}}\n");
            }
            AST::Variable(_, name) => self.buf.push_str(name),
            AST::VarDeclaration(_, name, val) => {
                self.buf.push_str(&format!("Ref<Value> {} = ", name));
                self.comp(val)?;
            }
            AST::Function { name, args, body, .. } => {
                let dbg_name = name.clone().unwrap_or_else(|| self.uuid());
                let var = self.uuid();
                match name.as_ref() {
                    Some(name) => self.buf.push_str(&format!("Ref<Value> {} = ", name)),
                    None => {},
                }
                self.buf.push_str(&format!("({{ {} *{} = new {}([&](", self.fn_type, var, self.fn_type));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.buf.push_str(", ");
                    }
                    self.buf.push_str(&format!("Ref<Value> {}", arg));
                }
                self.buf.push_str(") -> Ref<Value>");
                self.comp(body)?;
                self.buf.push_str(&format!("); Value::from_func(\"{}\", {}); }})", dbg_name, var));
            }
            _ => unimplemented!("Not implemented yet: {:?}", ast),
        };
        Ok(())
    }

    fn comp_builtin_call(&mut self, span: &Span, name: &str, args: &[Rc<AST>]) -> Result<()> {
        match name {
            "print" => {},
            _ => error!(span, "Unknown builtin function"),
        }

        let var = self.uuid();
        self.buf.push_str(&format!("({{ vector<Ref<Value>> {};\n", var));
        for arg in args.iter() {
            self.buf.push_str(&format!("  {}.push_back(", var));
            self.comp(arg)?;
            self.buf.push_str(");\n");
        }

        self.buf.push_str(&format!("{}(move({}), ", name, var));
        self.comp_loc(span)?;
        self.buf.push_str("); })");
        Ok(())
    }

    fn comp_loc(&mut self, span: &Span) -> Result<()> {
        self.buf.push_str(&format!("\"{}\"", span.0));
        Ok(())
    }
}