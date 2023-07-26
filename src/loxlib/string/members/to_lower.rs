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

pub struct ToLower {
    string: StringData,
}

impl ToLower {
    pub fn new(string: StringData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new(
            "toLower",
            Rc::new(Self { string }),
            true,
        )))
    }
}

impl LoxCallable for ToLower {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: Option<&Interpreter>, _: Vec<Literal>) -> Result<Literal, LoxResult> {
        Ok(Literal::Str(Rc::new(LoxString::new(self.string.borrow().to_lowercase()))))
    }
}
