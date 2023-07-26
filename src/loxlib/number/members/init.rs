use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{loxnatives::LoxNative, number::loxnumber::LoxNumber},
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Init;

impl Init {
    pub fn new() -> Literal {
        Literal::Native(Rc::new(LoxNative::new("init", Rc::new(Self {}), true)))
    }
}

impl LoxCallable for Init {
    fn arity(&self) -> usize {
        1
    }

    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let arg = args.get(0).unwrap();

        if arg.get_typename() != "String" {
            return Err(LoxResult::Message(format!(
                "Expected type 'String' but got {} for 'str' in Num(str)",
                arg.get_typename()
            )));
        }

        match arg.unwrap_str().parse::<f64>() {
            Ok(val) => Ok(Literal::Number(Rc::new(LoxNumber::new(val)))),
            Err(err) => Err(LoxResult::Message(format!(
                "Failed conversion to number: {err}"
            ))),
        }
    }
}
