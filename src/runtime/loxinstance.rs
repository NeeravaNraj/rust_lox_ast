use std::{fmt::Display, collections::HashMap};
use crate::{
    lexer::literal::Literal,
    lexer::token::Token,
    error::{LoxResult, loxerrorhandler::LoxErrorHandler, LoxErrorsTypes},
};

use std::cell::RefCell;

use super::loxclass::LoxClass;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    klass: LoxClass,
    fields: RefCell<HashMap<String, Literal>>,
    error_handler: LoxErrorHandler
}

impl LoxInstance {
    pub fn new(klass: &LoxClass) -> Self {
        Self {
            klass: klass.clone(),
            fields: RefCell::new(HashMap::new()),
            error_handler: LoxErrorHandler::new()
        }
    }

    pub fn get(&self, name: &Token) -> Result<Literal, LoxResult> {
        if self.fields.borrow().contains_key(&name.lexeme) {
            return Ok(self.fields.borrow().get(&name.lexeme).unwrap().dup())
        } 

        if let Some(m) = self.klass.find_method(&name.lexeme) {
            return Ok(m)
        }

        Err(self.error_handler.error(
            name, 
            LoxErrorsTypes::Runtime(format!("Undefined propert '{}'", name.lexeme))
        ))
    }

    pub fn set(&self, name: &Token, val: Literal) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), val);
    }
}

impl<'a> Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instance {}", self.klass)
    }
}
