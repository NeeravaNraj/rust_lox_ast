use std::{rc::Rc, fmt::Debug};

use crate::{lexer::literal::Literal, errors::LoxError};
use super::interpreter::Interpreter;

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxError>;
}

#[derive(Clone)]
pub struct Func {
    pub func: Rc<dyn LoxCallable>
}

impl LoxCallable for Func {
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxError> {
        self.func.call(interpreter, args)
    }

    fn arity(&self) -> usize {
        self.func.arity()
    }
}

impl Debug for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Function>")
    }
}

impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.func, &other.func)
    }
}
