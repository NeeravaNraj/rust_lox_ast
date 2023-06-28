use std::{borrow::Borrow, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{LoxError, LoxErrorsTypes, RuntimeError::RuntimeErrorHandler},
    lexer::token::{Literal, Token},
};
#[derive(Clone)]
pub struct Environment {
    env: HashMap<String, Literal>,
    error_handler: RuntimeErrorHandler,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            error_handler: RuntimeErrorHandler::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosing(env: RefCell<Environment>) -> Self {
        Self {
            env: HashMap::new(),
            error_handler: RuntimeErrorHandler::new(),
            enclosing: Some(Rc::new(env)),
        }
    }

    pub fn define(&mut self, name: &Token, val: Literal) -> Result<(), LoxError> {
        if self.env.contains_key(&name.lexeme) {
            return Err(self.error_handler.error(
                name,
                LoxErrorsTypes::RuntimeError("Cannot redefine variable".to_string()),
            ));
        }
        self.env.insert(name.lexeme.to_string(), val);
        Ok(())
    }

    pub fn mutate(&mut self, name: &Token, val: Literal) -> Result<(), LoxError> {
        if !self.env.contains_key(&name.lexeme) {
            return Err(self.error_handler.error(
                name,
                LoxErrorsTypes::RuntimeError("Cannot mutate undefined variable".to_string()),
            ));
        }
        if self.enclosing.is_some() {
            let enc = self.enclosing.as_mut().unwrap();
            enc.borrow_mut().mutate(name, val)?;
            return Ok(());
        }
        self.env.insert(name.lexeme.to_string(), val);
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<Literal, LoxError> {
        if let Some(literal) = self.env.get(name.lexeme.as_str()) {
            return Ok(literal.clone());
        } else if let Some(enc) = &self.enclosing {
            return enc.borrow_mut().get(name);
        }
        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::RuntimeError("Undefined variable".to_string()),
        ))
    }
}
