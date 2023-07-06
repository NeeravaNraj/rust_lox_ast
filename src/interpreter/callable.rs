use std::{rc::Rc, fmt::{Debug, Display}};

use crate::{lexer::literal::Literal, error::*};
use super::interpreter::Interpreter;

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxResult>;
    fn to_string(&self) -> String;
}

#[derive(Clone)]
pub struct Callable{
    pub func: Rc<dyn LoxCallable>
}

impl LoxCallable for Callable{
    fn call(&self, interpreter: &Interpreter, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        self.func.call(interpreter, args)
    }

    fn arity(&self) -> usize {
        self.func.arity()
    }

    fn to_string(&self) -> String {
        self.func.to_string()
    }
}

impl Debug for Callable{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.func)
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.func, &other.func)
    }
}

impl Display for dyn LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

