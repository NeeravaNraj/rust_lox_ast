use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{loxnatives::LoxNative, string::loxstring::LoxString},
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Init;

impl Init {
    pub fn new() -> Literal {
        Literal::Native(Rc::new(LoxNative::new("push", Rc::new(Self {}), false)))
    }
}

impl LoxCallable for Init {
    fn arity(&self) -> usize {
        1
    }

    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let string = args.get(0).unwrap().get_value();
        Ok(Literal::Str(Rc::new(LoxString::new(string))))
    }
}
