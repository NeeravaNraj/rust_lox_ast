use super::{
    callable::LoxCallable, environment::Environment, interpreter::Interpreter,
    loxinstance::LoxInstance,
};
use crate::{
    error::*,
    lexer::literal::Literal,
    lexer::token::*,
    parser::{expr::LambdaExpr, stmt::*},
};
use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    pub is_static: bool,
    is_initializer: bool,
    name: Option<Token>,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Rc<Stmt>>>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        decl: &FunctionStmt,
        env: &Rc<RefCell<Environment>>,
        is_initializer: bool,
        is_static: bool,
    ) -> Self {
        Self {
            name: Some(decl.name.dup()),
            params: Rc::clone(&decl.params),
            body: decl.body.clone(),
            closure: Rc::clone(env),
            is_initializer,
            is_static
        }
    }

    pub fn new_lambda(decl: &LambdaExpr, env: &Rc<RefCell<Environment>>) -> Self {
        Self {
            name: None,
            params: Rc::clone(&decl.params),
            body: Rc::clone(&decl.body),
            closure: Rc::clone(env),
            is_initializer: false,
            is_static: false
        }
    }

    pub fn bind(&self, instance: Rc<LoxInstance>) -> Result<Rc<Self>, LoxResult> {
        let mut env = Environment::new_enclosing(self.closure.clone());
        env.define(&Token::this(), Literal::Instance(instance))?;
        Ok(Rc::new(LoxFunction {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
            closure: Rc::new(RefCell::new(env)),
            is_initializer: self.is_initializer,
            is_static: self.is_static
        }))
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_some() {
            return write!(f, "<Fn {}>", self.name.as_ref().unwrap().lexeme);
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

        if self.is_initializer {
            return Ok(self.closure.borrow().get_at(0, &Token::this())?);
        }
        Ok(Literal::None)
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}
