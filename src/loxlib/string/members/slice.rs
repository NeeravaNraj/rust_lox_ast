use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{
        loxnatives::LoxNative,
        string::{loxstring::LoxString, StringData},
    },
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Slice {
    string: StringData,
}

impl Slice {
    pub fn new(string: StringData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new(
            "slice",
            Rc::new(Self { string }),
            true,
        )))
    }
}

impl LoxCallable for Slice {
    fn arity(&self) -> usize {
        2
    }

    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let start = args.get(0).unwrap();
        let end = args.get(1).unwrap();
        if start.get_typename() != "Number" {
            return Err(LoxResult::Message(format!(
                "Expected type 'Number' for start in Str.slice(start, end) got '{}'",
                start.get_typename()
            )));
        }

        if end.get_typename() != "Number" {
            return Err(LoxResult::Message(format!(
                "Expected type 'Number' for end in Str.slice(start, end) got '{}'",
                end.get_typename()
            )));
        }

        let start = start.unwrap_number() as usize;
        let end = end.unwrap_number() as usize;
        let len = self.string.borrow().len();

        if end <= start || start > len {
            return Ok(Literal::Str(Rc::new(LoxString::new(String::from("")))));
        }

        if start < len && end > len {
            let str = &self.string.borrow()[start..len];
            return Ok(Literal::Str(Rc::new(LoxString::new(str.to_string()))));
        }
        let str = &self.string.borrow()[start..end];
        Ok(Literal::Str(Rc::new(LoxString::new(str.to_string()))))
    }
}
