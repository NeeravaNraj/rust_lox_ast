use std::rc::Rc;

use crate::{
    error::*, lexer::literal::Literal, loxlib::{loxnatives::LoxNative, array::ArrayData},
    runtime::callable::LoxCallable, runtime::interpreter::Interpreter,
};

pub struct Len {
    array: ArrayData
}

impl Len {
    pub fn new(array: ArrayData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new("len", Rc::new(Self { array }), true)))
    }
}

impl LoxCallable for Len {
    fn call(&self, _: Option<&Interpreter>, _: Vec<Literal>) -> Result<Literal, LoxResult> {
        Ok(Literal::Number(self.array.borrow().len() as f64))
    }

    fn arity(&self) -> usize {
        0
    }
}
