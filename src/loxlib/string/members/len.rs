use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{loxnatives::LoxNative, string::StringData, number::loxnumber::LoxNumber},
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Len {
    string: StringData
}

impl Len {
    pub fn new(string: StringData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new("len", Rc::new(Self { string }), true)))
    }
}

impl LoxCallable for Len {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: Option<&Interpreter>, _: Vec<Literal>) -> Result<Literal, LoxResult> {
        Ok(Literal::Number(Rc::new(LoxNumber::new(self.string.borrow().len() as f64))))
    }
}
