use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{loxerrorhandler::LoxErrorHandler, LoxErrorsTypes, LoxResult},
    lexer::literal::*,
    lexer::token::Token,
};
#[derive(Clone)]
pub struct Environment {
    pub loop_started: bool,
    pub continue_encountered: bool,
    pub id: usize,
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
            continue_encountered: false,
            natives: HashMap::new(),
            id: 0,
        }
    }

    pub fn new_enclosing(env: Rc<RefCell<Environment>>) -> Self {
        let id = env.borrow().id + 1;
        Self {
            env: HashMap::new(),
            error_handler: LoxErrorHandler::new(),
            enclosing: Some(env),
            continue_encountered: false,
            loop_started: false,
            natives: HashMap::new(),
            id,
        }
    }

    pub fn define_native(&mut self, name: &Token, val: Literal) -> Result<(), LoxResult> {
        if self.env.contains_key(&name.lexeme) {
            return Err(self.error_handler.error(
                name,
                LoxErrorsTypes::Runtime("Cannot redefine variable".to_string()),
            ));
        }
        self.env.insert(name.lexeme.to_string(), val);
        self.natives.insert(name.lexeme.to_string(), ());
        Ok(())
    }

    pub fn define(&mut self, name: &Token, val: Literal) -> Result<(), LoxResult> {
        if self.env.contains_key(&name.lexeme) {
            return Err(self.error_handler.error(
                name,
                LoxErrorsTypes::Runtime("Cannot redefine variable".to_string()),
            ));
        }
        self.env.insert(name.lexeme.to_string(), val);
        Ok(())
    }

    pub fn mutate(&mut self, name: &Token, val: Literal) -> Result<(), LoxResult> {
        if self.natives.contains_key(&name.lexeme) {
            return Err(self.error_handler.error(
                name,
                LoxErrorsTypes::Runtime("Cannot overwrite language feature".to_string()),
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
            LoxErrorsTypes::Runtime("Cannot mutate undefined variable".to_string()),
        ))
    }

    pub fn get(&self, name: &Token) -> Result<Literal, LoxResult> {
        if let Some(literal) = self.env.get(name.lexeme.as_str()) {
            return Ok(literal.clone());
        } else if let Some(enc) = &self.enclosing {
            return enc.borrow().get(name);
        }
        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::Runtime("Undefined variable".to_string()),
        ))
    }
}
