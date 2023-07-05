use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{LoxError, LoxErrorsTypes, loxerrorhandler::LoxErrorHandler},
    lexer::token::Token,
    lexer::literal::*
};
#[derive(Clone)]
pub struct Environment {
    pub loop_started: bool,
    pub break_encountered: bool,
    pub continue_encountered: bool,
    env: HashMap<String, Literal>,
    natives: HashMap<String, ()>,
    error_handler: LoxErrorHandler,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            error_handler: LoxErrorHandler::new(),
            enclosing: None,
            loop_started: false,
            break_encountered: false,
            continue_encountered: false,
            natives: HashMap::new()
        }
    }

    pub fn new_enclosing(env: Rc<RefCell<Environment>>) -> Self {
        Self {
            env: HashMap::new(),
            error_handler: LoxErrorHandler::new(),
            enclosing: Some(env),
            loop_started: false,
            continue_encountered: false,
            break_encountered: false,
            natives: HashMap::new()
        }
    }

    pub fn define_native(&mut self, name: &Token, val: Literal) -> Result<(), LoxError> {
        if self.env.contains_key(&name.lexeme) {
            return Err(self.error_handler.error(
                name,
                LoxErrorsTypes::RuntimeError("Cannot redefine variable".to_string()),
            ));
        }
        self.env.insert(name.lexeme.to_string(), val);
        self.natives.insert(name.lexeme.to_string(), ());
        Ok(())
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
        if self.natives.contains_key(&name.lexeme) {
            return Err(self.error_handler.error(
                name,
                LoxErrorsTypes::RuntimeError("Cannot overwrite language feature".to_string()),
            ));
        }
        if self.env.contains_key(&name.lexeme) {
            self.env.insert(name.lexeme.to_string(), val);
            return Ok(());
        } else if self.enclosing.is_some() {
            let enc = self.enclosing.as_mut().unwrap();
            enc.borrow_mut().mutate(name, val)?;
            return Ok(());
        }

        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::RuntimeError("Cannot mutate undefined variable".to_string()),
        ))
    }

    pub fn get(&self, name: &Token) -> Result<Literal, LoxError> {
        if let Some(literal) = self.env.get(name.lexeme.as_str()) {
            return Ok(literal.clone());
        } else if let Some(enc) = &self.enclosing {
            return enc.borrow().get(name);
        }
        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::RuntimeError("Undefined variable".to_string()),
        ))
    }
}
