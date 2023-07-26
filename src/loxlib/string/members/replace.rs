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

pub struct Replace {
    string: StringData,
}

impl Replace {
    pub fn new(string: StringData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new(
            "replace",
            Rc::new(Self { string }),
            true,
        )))
    }
}

impl LoxCallable for Replace {
    fn arity(&self) -> usize {
        2
    }

    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let text = args.get(0).unwrap();
        let value = args.get(1).unwrap();
        if text.get_typename() != "String" {
            return Err(LoxResult::Message(format!(
                "Expected type 'String' for 'text' in Str.replace(text, value) got '{}'",
                text.get_typename()
            )));
        }

        if value.get_typename() != "String" {
            return Err(LoxResult::Message(format!(
                "Expected type 'String' for 'value' in Str.replace(text, end) got '{}'",
                value.get_typename()
            )));
        }
        let text = text.unwrap_str();
        let value = value.unwrap_str();
        let str = self.string.borrow_mut().replacen(&text, &value, 1);
        Ok(Literal::Str(Rc::new(LoxString::new(str))))
    }
}
