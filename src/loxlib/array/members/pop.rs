use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{array::ArrayData, loxnatives::LoxNative},
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Pop {
    array: ArrayData,
}

impl Pop {
    pub fn new(array: ArrayData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new("pop", Rc::new(Self { array }), true)))
    }
}

impl LoxCallable for Pop {
    fn call(&self, _: Option<&Interpreter>, _: Vec<Literal>) -> Result<Literal, LoxResult> {
        if let Some(val) = self.array.borrow_mut().pop() {
            Ok(val)
        } else {
            return Err(LoxResult::Message(
                "Cannot pop from empty array".to_string(),
            ));
        }
    }

    fn arity(&self) -> usize {
        0
    }
}
