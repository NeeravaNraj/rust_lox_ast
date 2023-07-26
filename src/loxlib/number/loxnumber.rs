use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::runtime::loxinstance::InstanceField;
use crate::{
    error::LoxResult,
    lexer::{literal::Literal, token::Token},
    runtime::{loxclass::LoxClass, loxinstance::LoxInstance},
};

use super::NumberData;
use super::number_class_member::NumberMembers;
#[derive(Debug, Clone, PartialEq)]
struct NumberClass {
    klass: LoxClass,
    fields: Rc<RefCell<HashMap<String, InstanceField>>>,
}

impl NumberClass {
    fn new(num: &NumberData) -> Self {
        let members = NumberMembers::new(num.clone());
        Self {
            klass: LoxClass::new(
                "Number",
                members.get_methods(),
                members.get_statics(),
                members.get_fields(),
            ),
            fields: Rc::new(RefCell::new(members.get_fields())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxNumber {
    pub num: NumberData,
    num_class: Rc<NumberClass>,
    num_inst: Rc<LoxInstance>
}

impl LoxNumber {
    pub fn new(num: f64) -> Self {
        let num = Rc::new(RefCell::new(num));
        let cl = NumberClass::new(&num);
        let inst = LoxInstance::new(&cl.klass, cl.fields.clone());
        Self {
            num,
            num_inst: Rc::new(inst),
            num_class: Rc::new(cl),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Literal, LoxResult> {
        return self.num_inst.get(name, &self.num_inst)
    }
}
