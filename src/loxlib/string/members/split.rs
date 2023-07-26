use std::rc::Rc;

use crate::{
    error::*,
    lexer::literal::Literal,
    loxlib::{
        array::loxarray::LoxArray,
        loxnatives::LoxNative,
        string::{loxstring::LoxString, StringData},
    },
    runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Split {
    string: StringData,
}

impl Split {
    pub fn new(string: StringData) -> Literal {
        Literal::Native(Rc::new(LoxNative::new(
            "split",
            Rc::new(Self { string }),
            true,
        )))
    }
}

impl LoxCallable for Split {
    fn arity(&self) -> usize {
        1
    }

    fn call(&self, _: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let arg = args.get(0).unwrap();
        if arg.get_typename() != "String" {
            return Err(LoxResult::Message(format!(
                "Expected type 'String' for 'at' in Str.split(at) got '{}'",
                arg.get_typename()
            )));
        }

        let mut arr: Vec<Literal> = Vec::new();
        for s in self
            .string
            .borrow()
            .split(&arg.unwrap_str())
            .filter(|x| !x.is_empty())
        {
            arr.push(Literal::Str(Rc::new(LoxString::new(s.to_string()))))
        }
        Ok(Literal::Array(Rc::new(LoxArray::new(arr))))
    }
}
