use std::rc::Rc;

use crate::{
    error::*, lexer::literal::Literal, runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

use super::string::loxstring::LoxString;

pub struct TypeOf;

impl LoxCallable for TypeOf {
    fn arity(&self) -> usize {
        1
    }

    fn call(
        &self,
        _interpreter: Option<&Interpreter>,
        args: Vec<Literal>,
    ) -> Result<Literal, LoxResult> {
        Ok(Literal::Str(Rc::new(LoxString::new(
            args.get(0).unwrap().get_typename(),
        ))))
    }
}
