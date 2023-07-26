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

pub struct ToUpper {
    string: StringData,
}

impl ToUpper {
    pub fn new(string: StringData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new(
            "toUpper",
            Rc::new(Self { string }),
            true,
        )))
    }
}

impl LoxCallable for ToUpper {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: Option<&Interpreter>, _: Vec<Literal>) -> Result<Literal, LoxResult> {
        Ok(Literal::Str(Rc::new(LoxString::new(self.string.borrow().to_uppercase()))))
    }
}
