use std::{rc::Rc, cell::RefCell};

pub mod loxstring;
pub mod string_class_member;
pub mod members;
type StringData = Rc<RefCell<String>>;
