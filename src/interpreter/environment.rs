use std::collections::HashMap;

use crate::{lexer::token::{Literal, Token}, errors::{LoxError, RuntimeError::{RuntimeErrorHandler}, LoxErrorsTypes}};

pub struct Environment {
    env: HashMap<String, Literal>,
    error_handler: RuntimeErrorHandler
}

impl Environment {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            error_handler: RuntimeErrorHandler::new()
        }
    }

    pub fn define(&mut self, name: &String, val: Literal) {
        self.env.insert(name.to_string(), val);
    }

    pub fn get(&self, name: &Token) -> Result<Literal, LoxError> {
        if let Some(literal) = self.env.get(name.lexeme.as_str()) {
            return Ok(literal.clone());
        } else {
            Err(self.error_handler.error(name, LoxErrorsTypes::RuntimeError("Undefined variable".to_string())))
        }
    }
}

