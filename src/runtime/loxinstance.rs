use crate::{
    error::{loxerrorhandler::LoxErrorHandler, LoxErrorsTypes, LoxResult},
    lexer::literal::Literal,
    lexer::token::Token,
};
use std::{collections::HashMap, fmt::Display};

use std::cell::RefCell;
use std::rc::Rc;

use super::loxclass::LoxClass;

#[derive(Debug, Clone, PartialEq)]
pub struct InstanceField {
    pub value: Literal,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    klass: LoxClass,
    fields: Rc<RefCell<HashMap<String, InstanceField>>>,
    error_handler: LoxErrorHandler,
    pub this: RefCell<bool>,
}

impl LoxInstance {
    pub fn new(klass: &LoxClass, fields: Rc<RefCell<HashMap<String, InstanceField>>>) -> Self {
        Self {
            klass: klass.clone(),
            fields,
            error_handler: LoxErrorHandler::new(),
            this: RefCell::new(false),
        }
    }

    pub fn get(&self, name: &Token, this: &Rc<LoxInstance>) -> Result<Literal, LoxResult> {
        if self.fields.borrow().contains_key(&name.lexeme) {
            if !(self
                .fields
                .borrow()
                .get(&name.lexeme)
                .as_ref()
                .unwrap()
                .is_public
                || self.this.borrow().eq(&true))
            {
                return Err(self.error_handler.error(
                    name,
                    LoxErrorsTypes::ReferenceError("Cannot get private property".to_string()),
                ));
            }
            *self.this.borrow_mut() = false;
            return Ok(self.fields.borrow().get(&name.lexeme).unwrap().value.dup());
        }

        if let Some(m) = self.klass.find_method(&name.lexeme) {
            match m {
                Literal::Func(method) => {
                    if method.is_static {
                        return Err(self.error_handler.error(
                            name,
                            LoxErrorsTypes::Runtime(
                                "Cannot call static method from instantiated class".to_string(),
                            ),
                        ));
                    }
                    if !(method.is_pub || self.this.borrow().eq(&true)) {
                        return Err(self.error_handler.error(
                            name,
                            LoxErrorsTypes::Runtime("Cannot call private method".to_string()),
                        ));
                    }
                    return Ok(Literal::Func(method.bind(this.clone())?))
                }
                Literal::Native(method) =>  return Ok(Literal::Native(method.clone())),
                _ => {
                    panic!("tried to bind 'this' to non function literal {m:?}")
                }
            }
        }

        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::Runtime("Undefined property".to_string()),
        ))
    }

    pub fn set(&self, name: &Token, val: Literal) -> Result<(), LoxResult> {
        if self.fields.borrow().contains_key(&name.lexeme) {
            if !(self
                .fields
                .borrow()
                .get(&name.lexeme)
                .as_ref()
                .unwrap()
                .is_public
                || self.this.borrow().eq(&true))
            {
                return Err(self.error_handler.error(
                    name,
                    LoxErrorsTypes::ReferenceError("Cannot set private property".to_string()),
                ));
            }
            *self.this.borrow_mut() = false;
            self.fields
                .borrow_mut()
                .get_mut(&name.lexeme)
                .unwrap()
                .value = val.dup();
            return Ok(());
        }
        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::ReferenceError("Cannot access undefined vairable".to_string()),
        ))
    }
}

impl<'a> Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Instance {}>", self.klass.name)
    }
}
