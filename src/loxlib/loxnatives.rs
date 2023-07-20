use std::time::SystemTime;
use std::{rc::Rc, fmt::{Debug, Display}};

use crate::{
    error::*, lexer::literal::Literal, runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &Interpreter, _args: Vec<Literal>) -> Result<Literal, LoxResult> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => Ok(Literal::Number(time.as_secs_f64())),
            Err(err) => Err(LoxResult::Error(LoxError::system_error(
                format!(
                    "Clock return invalid duration: {}",
                    err.duration().as_secs_f64()
                )
                .as_str(),
            ))),
        }
    }
}

pub struct LoxNative {
    pub name: String,
    pub native: Rc<dyn LoxCallable>,
}

impl LoxNative {
    pub fn new(name: &str, native: Rc<dyn LoxCallable>) -> Self {
        Self {
            name: name.to_string(),
            native
        }
    }
}

impl PartialEq for LoxNative {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            Rc::as_ptr(&self.native) as *const (),
            Rc::as_ptr(&other.native) as *const (),
        )
    }
}

impl Debug for LoxNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Native-fn>")
    }
}

impl Display for LoxNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Fn {}>", self.name)
    }
}
