use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, fmt::Display};

use super::loxinstance::InstanceField;
use super::{callable::LoxCallable, interpreter::Interpreter, loxinstance::LoxInstance};
use crate::error::loxerrorhandler::LoxErrorHandler;
use crate::{
    error::{LoxErrorsTypes, LoxResult},
    lexer::literal::Literal,
    lexer::token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: String,
    methods: HashMap<String, Literal>,
    static_fields: RefCell<HashMap<String, Literal>>,
    other_fields: Rc<RefCell<HashMap<String, InstanceField>>>,
    error_handler: LoxErrorHandler,
}

impl LoxClass {
    pub fn new(
        name: &str,
        methods: HashMap<String, Literal>,
        static_fields: HashMap<String, Literal>,
        other_fields: HashMap<String, InstanceField>,
    ) -> Self {
        Self {
            name: name.to_string(),
            methods,
            static_fields: RefCell::new(static_fields),
            other_fields: Rc::new(RefCell::new(other_fields)),
            error_handler: LoxErrorHandler,
        }
    }

    pub fn find_method(&self, name: &String) -> Option<Literal> {
        if let Some(m) = self.methods.get(name) {
            return Some(m.clone());
        }

        None
    }

    pub fn get(&self, name: &Token, class: &Rc<LoxClass>) -> Result<Literal, LoxResult> {
        if self.static_fields.borrow().contains_key(&name.lexeme) {
            return Ok(self.static_fields.borrow().get(&name.lexeme).unwrap().dup())
        }

        if let Some(method) = self.find_method(&name.lexeme) {
            if let Literal::Func(f) = &method {
                if f.is_static {
                    return Ok(Literal::Func(f.bind_static(class.clone())?))
                }
                return Err(self.error_handler.error(
                    name,
                    LoxErrorsTypes::ReferenceError(
                        "Trying to access non static property".to_string(),
                    ),
                ));
            } else {
                panic!("non function literal {method:?}")
            }
        }
        return Err(self.error_handler.error(
            name,
            LoxErrorsTypes::ReferenceError("Trying to access undefined property".to_string()),
        ));
    }

    pub fn set(&self, name: &Token, val: Literal) -> Result<(), LoxResult> {
        if self.static_fields.borrow().contains_key(&name.lexeme) {
            *self
                .static_fields
                .borrow_mut()
                .get_mut(&name.lexeme)
                .unwrap() = val.dup();
            return Ok(());
        }
        Err(self.error_handler.error(
            name,
            LoxErrorsTypes::ReferenceError("Cannot access undefined vairable".to_string()),
        ))
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Class {}>", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, interpreter: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let instance = Rc::new(LoxInstance::new(self, self.other_fields.clone()));
        let initializer = self.find_method(&"init".to_string());
        if let Some(init) = initializer {
            match init {
                Literal::Func(func) => {
                    func.bind(instance.clone())?.call(interpreter, args)?;
                }
                Literal::Native(n) => {
                    return n.native.call(interpreter, args)
                }
                _ => {
                    panic!("found non function literal in constructor")
                }
            }
        }
        Ok(Literal::Instance(instance.clone()))
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method(&"init".to_string());
        if let Some(init) = initializer {
            match init {
                Literal::Func(func) => return func.arity(),
                Literal::Native(n) => return n.native.arity(),
                _ => {
                    panic!("found non function literal in constructor arity")
                }
            }
        }
        0
    }
}
