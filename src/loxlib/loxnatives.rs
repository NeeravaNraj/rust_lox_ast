use crate::runtime::callable::LoxCallable;
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

pub struct LoxNative {
    pub name: String,
    pub native: Rc<dyn LoxCallable>,
    pub check_arity: bool,
}

impl LoxNative {
    pub fn new(name: &str, native: Rc<dyn LoxCallable>, check_arity: bool) -> Self {
        Self {
            name: name.to_string(),
            native,
            check_arity,
        }
    }
}

impl PartialEq for LoxNative {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            Rc::as_ptr(&self.native) as *const (),
            Rc::as_ptr(&other.native) as *const (),
        )
    }
}

impl Debug for LoxNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Native-fn>")
    }
}

impl Display for LoxNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Fn {}>", self.name)
    }
}
