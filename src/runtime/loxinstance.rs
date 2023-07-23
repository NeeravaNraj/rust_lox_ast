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
pub struct LoxInstance {
    klass: LoxClass,
    fields: RefCell<HashMap<String, Literal>>,
    error_handler: LoxErrorHandler,
}

impl LoxInstance {
    pub fn new(klass: &LoxClass) -> Self {
        Self {
            klass: klass.clone(),
            fields: RefCell::new(HashMap::new()),
            error_handler: LoxErrorHandler::new(),
        }
    }

    pub fn get(&self, name: &Token, this: &Rc<LoxInstance>) -> Result<Literal, LoxResult> {
        if self.fields.borrow().contains_key(&name.lexeme) {
            return Ok(self.fields.borrow().get(&name.lexeme).unwrap().dup());
        }

        if let Some(m) = self.klass.find_method(&name.lexeme) {
            if let Literal::Func(method) = m {
                if method.is_static {
                    return Err(self.error_handler.error(
                        name,
                        LoxErrorsTypes::Runtime(
                            "Cannot call static method from instantiated class".to_string(),
                        ),
                    ));
                }
                return Ok(Literal::Func(method.bind(this.clone())?));
            } else {
                panic!("tried to bind 'this' to non function literal {m:?}")
            }
        }

        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::Runtime("Undefined property".to_string()),
        ))
    }

    pub fn set(&self, name: &Token, val: Literal) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), val);
    }
}

impl<'a> Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Instance {}>", self.klass.name)
    }
}
