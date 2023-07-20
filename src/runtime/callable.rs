use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use super::interpreter::Interpreter;
use crate::{error::*, lexer::literal::Literal};

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxResult>;
}

#[derive(Clone)]
pub struct Callable {
    pub func: Rc<dyn LoxCallable>,
}

impl LoxCallable for Callable {
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        self.func.call(interpreter, args)
    }

    fn arity(&self) -> usize {
        self.func.arity()
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.func)
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            &*self.func as *const dyn LoxCallable as *const u8,
            &*other.func as *const dyn LoxCallable as *const u8,
        )
    }
}

impl Display for dyn LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.func.to_string())
    }
}
