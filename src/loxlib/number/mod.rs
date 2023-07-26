use std::{rc::Rc, cell::RefCell};

pub mod loxnumber;
pub mod members;
pub mod number_class_member;
type NumberData = Rc<RefCell<f64>>;
