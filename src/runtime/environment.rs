use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{loxerrorhandler::LoxErrorHandler, LoxErrorsTypes, LoxResult},
    lexer::literal::*,
    lexer::token::Token,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub continue_encountered: bool,
    pub id: usize,
    pub env: HashMap<String, Literal>,
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
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().mutate(name, val)?;
            return Ok(());
        }

        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::Runtime("Cannot mutate undefined variable".to_string()),
        ))
    }

    pub fn mutate_at(&mut self, distance: usize, name: &Token, val: Literal) -> Result<(), LoxResult> {
        if distance == 0 {
            self.mutate(name, val)?;
            return Ok(());
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .mutate_at(distance - 1, name, val)
        }
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

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Literal, LoxResult> {
        // println!("{} {}", name.lexeme, name.line);
        if distance == 0 {
            if let Some(var) = self.env.get(&name.lexeme) {
                return Ok(var.dup());
            }
            println!("{:?}", self.env.keys());
            return Err(self.error_handler.error(
                name, 
                LoxErrorsTypes::ReferenceError("Undefined variable".to_string())
            ));
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get_at(distance - 1, name)
        }
    }
}
