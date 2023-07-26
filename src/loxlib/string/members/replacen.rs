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

pub struct Replacen {
    string: StringData,
}

impl Replacen {
    pub fn new(string: StringData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new(
            "replace",
            Rc::new(Self { string }),
            true,
        )))
    }
}

impl LoxCallable for Replacen {
    fn arity(&self) -> usize {
        3
    }

    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let text = args.get(0).unwrap();
        let value = args.get(1).unwrap();
        let n = args.get(2).unwrap();
        if text.get_typename() != "String" {
            return Err(LoxResult::Message(format!(
                "Expected type 'String' for 'text' in Str.replacen(text, value, times) got '{}'",
                text.get_typename()
            )));
        }

        if value.get_typename() != "String" {
            return Err(LoxResult::Message(format!(
                "Expected type 'String' for 'value' in Str.replacen(text, value, times) got '{}'",
                value.get_typename()
            )));
        }

        if n.get_typename() != "Number" {
            return Err(LoxResult::Message(format!(
                "Expected type 'Number' for 'times' in Str.replacen(text, value, times) got '{}'",
                value.get_typename()
            )));
        }
        let text = text.unwrap_str();
        let value = value.unwrap_str();
        let str = self.string.borrow_mut().replacen(&text, &value, 1);
        Ok(Literal::Str(Rc::new(LoxString::new(str))))
    }
}
