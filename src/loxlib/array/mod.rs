use std::{rc::Rc, cell::RefCell};

use crate::lexer::literal::Literal;

pub mod loxarray;
pub mod array_class_members;
pub mod members;
type ArrayData = Rc<RefCell<Vec<Literal>>>;
