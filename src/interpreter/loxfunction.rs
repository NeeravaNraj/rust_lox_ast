use crate::{
    parser::stmt::*,
    lexer::literal::Literal,
    lexer::token::*,
    error::*,
};
use std::rc::Rc;
use super::{
    callable::LoxCallable,
    interpreter::Interpreter, environment::Environment,
};

pub struct LoxFunction {
    name: Token,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Box<Stmt>>>
}

impl LoxFunction {
    pub fn new(decl: &FunctionStmt) -> Self {
        Self {
            name: decl.name.dup(),
            params: Rc::clone(&decl.params),
            body: Rc::clone(&decl.body)
        }
    }

    pub fn to_string(&self) -> String {
        return format!("<Fn {}>", self.name)
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let mut env = Environment::new_enclosing(interpreter.globals.clone());
        for (i, d) in self.params.iter().enumerate() {
            if let Some(val) = args.get(i) {
                env.define(&d, val.dup())?;
            }
        }
        // for arg in args {
        //     arg.print_value();
        // }
        if let Err(ret_val) = interpreter.execute_block(&self.body, env) {
            match ret_val {
                LoxResult::LoxReturn(value) => return Ok(value),
                _ => return Err(ret_val)
            }
        }
        Ok(Literal::None)
    }

    fn arity(&self) -> usize {
        return self.params.len()
    }

    fn to_string(&self) -> String {
        self.to_string()
    }
}
