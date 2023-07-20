use super::{callable::LoxCallable, environment::Environment, interpreter::Interpreter};
use crate::{error::*, lexer::literal::Literal, lexer::token::*, parser::{stmt::*, expr::LambdaExpr}};
use std::{fmt::Display, rc::Rc, cell::RefCell};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    name: Option<Token>,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Rc<Stmt>>>,
    closure: Rc<RefCell<Environment>>
}

impl LoxFunction {
    pub fn new(decl: &FunctionStmt, env: &Rc<RefCell<Environment>>) -> Self {
        Self {
            name: Some(decl.name.dup()),
            params: Rc::clone(&decl.params),
            body: decl.body.clone(),
            closure: Rc::clone(env)
        }
    }

    pub fn new_lambda(decl: &LambdaExpr, env: &Rc<RefCell<Environment>>) -> Self {
        Self {
            name: None,
            params: Rc::clone(&decl.params),
            body: Rc::clone(&decl.body),
            closure: Rc::clone(env)
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_some() {
            return write!(f, "<Fn {}>", self.name.as_ref().unwrap().lexeme)
        } 
        write!(f, "<Fn Lamba>")
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let mut env = Environment::new_enclosing(self.closure.clone());
        for (i, d) in self.params.iter().enumerate() {
            if let Some(val) = args.get(i) {
                env.define(d, val.dup())?;
            }
        }
        if let Err(ret_val) = interpreter.execute_block(&self.body, env) {
            match ret_val {
                LoxResult::Return(value) => return Ok(value),
                _ => return Err(ret_val),
            }
        }
        Ok(Literal::None)
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    // fn to_string(&self) -> String {
    //     if self.name.is_some() {
    //         return format!("<Fn {}>", self.name.as_ref().unwrap().lexeme)
    //     }
    //     String::from("<Lambda>")
    // }
}
