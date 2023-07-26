use std::rc::Rc;

use crate::{
    error::*, lexer::literal::Literal, loxlib::{loxnatives::LoxNative, array::ArrayData},
    runtime::callable::LoxCallable, runtime::interpreter::Interpreter,
};

pub struct Push {
    array: ArrayData
}

impl Push {
    pub fn new(array: ArrayData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new("push", Rc::new(Self { array }), true)))
    }
}

impl LoxCallable for Push {
    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        self.array.borrow_mut().push(args.get(0).unwrap().dup());
        Ok(Literal::None)
    }

    fn arity(&self) -> usize {
        1
    }
}
