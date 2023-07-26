use std::rc::Rc;

use crate::{runtime::loxinstance::LoxInstance, error::LoxResult, lexer::literal::Literal};

pub trait NativeMethod {
    fn bind(&self, inst: Rc<LoxInstance>) -> Result<Literal, LoxResult>;
}
