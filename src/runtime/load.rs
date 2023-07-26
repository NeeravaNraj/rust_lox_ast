use std::{cell::RefCell, rc::Rc};

use crate::{
    error::LoxResult,
    lexer::{literal::Literal, token::Token, tokentype::TokenType},
    loxlib::{
        array::array_class_members::ArrayMembers, clock::Clock, input::Input,
        loxnatives::LoxNative, print::Print, string::string_class_member::StringMembers,
    },
};

use super::{environment::Environment, loxclass::LoxClass};

pub fn load(env: Rc<RefCell<Environment>>) -> Result<(), LoxResult> {
    let array_members = ArrayMembers::new(Rc::new(RefCell::new(Vec::new())));
    let string_members = StringMembers::new(Rc::new(RefCell::new(String::new())));
    let natives = [
        (
            Token::new(TokenType::DefFn, "clock".to_string(), None, 0),
            Literal::Native(Rc::new(LoxNative::new("clock", Rc::new(Clock {}), true))),
        ),
        (
            Token::new(TokenType::DefFn, "print".to_string(), None, 0),
            Literal::Native(Rc::new(LoxNative::new("print", Rc::new(Print {}), false))),
        ),
        (
            Token::new(TokenType::DefFn, "input".to_string(), None, 0),
            Literal::Native(Rc::new(LoxNative::new("input", Rc::new(Input {}), false))),
        ),
        (
            Token::new(TokenType::Class, "Array".to_string(), None, 0),
            Literal::Native(Rc::new(LoxNative::new(
                "Array",
                Rc::new(LoxClass::new(
                    "Array",
                    array_members.get_methods(),
                    array_members.get_statics(),
                    array_members.get_fields(),
                )),
                false,
            ))),
        ),
        (
            Token::new(TokenType::Class, "Str".to_string(), None, 0),
            Literal::Native(Rc::new(LoxNative::new(
                "Str",
                Rc::new(LoxClass::new(
                    "Str",
                    string_members.get_methods(),
                    string_members.get_statics(),
                    string_members.get_fields(),
                )),
                false,
            ))),
        ),
    ];

    for (tok, native) in natives.iter() {
        env.borrow_mut().define_native(&tok, native.dup())?;
    }

    Ok(())
}
