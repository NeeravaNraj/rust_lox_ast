use std::{fmt::Display, collections::HashMap};
use std::rc::Rc;

use crate::{
    lexer::literal::Literal,
    error::LoxResult,
};
use super::{
    callable::LoxCallable,
    interpreter::Interpreter,
    loxinstance::LoxInstance,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Literal>
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, Literal>) -> Self {
        Self {
            name: name.to_string(),
            methods
        }
    }

    pub fn find_method(&self, name: &String) -> Option<Literal> {
        if let Some(m) = self.methods.get(name) {
            return Some(m.clone())
        }

        None
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Class {}>", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let instance = LoxInstance::new(self);
        Ok(Literal::Instance(Rc::new(instance)))
    }

    fn arity(&self) -> usize {
        0
    }
}
