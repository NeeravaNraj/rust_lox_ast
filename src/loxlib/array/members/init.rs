use std::rc::Rc;

use crate::{
    error::*, lexer::literal::Literal, loxlib::{loxnatives::LoxNative, array::loxarray::LoxArray},
    runtime::callable::LoxCallable, runtime::interpreter::Interpreter,
};

pub struct Init;

impl Init {
    pub fn new() -> Literal {
        Literal::Native(Rc::new(LoxNative::new("push", Rc::new(Self {}), false)))
    }
}

impl LoxCallable for Init {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let mut arr: Vec<Literal> = Vec::new();
        for arg in args.iter() {
            arr.push(arg.dup());
        }
        Ok(Literal::Array(Rc::new(LoxArray::new(arr))))
    }
}
