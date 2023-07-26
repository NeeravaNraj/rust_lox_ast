use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{
        loxnatives::LoxNative,
        string::{loxstring::LoxString, StringData},
    },
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Trim {
    string: StringData,
}

impl Trim {
    pub fn new(string: StringData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new(
            "trim",
            Rc::new(Self { string }),
            true,
        )))
    }
}

impl LoxCallable for Trim {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: Option<&Interpreter>, _: Vec<Literal>) -> Result<Literal, LoxResult> {
        Ok(Literal::Str(Rc::new(LoxString::new(
            self.string.borrow().trim().to_string(),
        ))))
    }
}
