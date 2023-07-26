use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{loxnatives::LoxNative, number::loxnumber::LoxNumber},
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct TryParse;

impl TryParse {
    pub fn new() -> Literal {
        Literal::Native(Rc::new(LoxNative::new("tryParse", Rc::new(Self {}), true)))
    }
}

impl LoxCallable for TryParse {
    fn arity(&self) -> usize {
        1
    }

    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let arg = args.get(0).unwrap();

        if arg.get_typename() != "String" {
            return Err(LoxResult::Message(format!(
                "Expected type 'String' but got {} for 'str' in Num.tryParse(str)",
                arg.get_typename()
            )));
        }

        match arg.unwrap_str().parse::<f64>() {
            Ok(val) => Ok(Literal::Number(Rc::new(LoxNumber::new(val)))),
            Err(_) => Ok(Literal::Bool(false)),
        }
    }
}
