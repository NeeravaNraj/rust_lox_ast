use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::runtime::loxinstance::InstanceField;
use crate::{
    error::LoxResult,
    lexer::{literal::Literal, token::Token},
    runtime::{loxclass::LoxClass, loxinstance::LoxInstance},
};

use super::ArrayData;
use super::array_class_members::ArrayMembers;

#[derive(Debug, Clone, PartialEq)]
struct ArrayClass {
    klass: LoxClass,
    fields: Rc<RefCell<HashMap<String, InstanceField>>>,
}

impl ArrayClass {
    fn new(arr: &ArrayData) -> Self {
        let members = ArrayMembers::new(arr.clone());
        Self {
            klass: LoxClass::new(
                "Array",
                members.get_methods(),
                members.get_statics(),
                members.get_fields(),
            ),
            fields: Rc::new(RefCell::new(members.get_fields())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxArray {
    pub array: ArrayData,
    arr_class: Rc<ArrayClass>,
    arr_inst: Rc<LoxInstance>
}

impl LoxArray {
    pub fn new(array: Vec<Literal>) -> Self {
        let arr = Rc::new(RefCell::new(array));
        let cl = ArrayClass::new(&arr);
        let inst = LoxInstance::new(&cl.klass, cl.fields.clone());
        Self {
            array: arr,
            arr_inst: Rc::new(inst),
            arr_class: Rc::new(cl),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Literal, LoxResult> {
        return self.arr_inst.get(name, &self.arr_inst)
    }
}
