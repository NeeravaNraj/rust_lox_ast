use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{array::ArrayData, loxnatives::LoxNative},
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Insert {
    array: ArrayData,
}

impl Insert {
    pub fn new(array: ArrayData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new("insert", Rc::new(Self { array }), true)))
    }
}

impl LoxCallable for Insert {
    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let index = args.get(0).expect("Array.insert index was null");
        let val = args.get(1).expect("Array.insert value was null");
        if index.get_typename() != "Number" {
            return Err(LoxResult::Message(
                "Index has to be number for Array.replace(index)".to_string(),
            ));
        }

        let index = index.unwrap_number();
        if index < 0.0 {
            return Err(LoxResult::Message(
                "Index has to be greater than 0 for Array.replace(index, value)".to_string(),
            ));
        }

        if index as usize > self.array.borrow().len() {
            return Err(LoxResult::Message("Index has to be less than or equal to array length for Array.replace(index, value)".to_string()));
        }
        self.array.borrow_mut().insert(index as usize, val.dup());
        Ok(Literal::None)
    }

    fn arity(&self) -> usize {
        2
    }
}
