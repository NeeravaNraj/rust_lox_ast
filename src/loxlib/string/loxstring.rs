use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::runtime::loxinstance::InstanceField;
use crate::{
    error::LoxResult,
    lexer::{literal::Literal, token::Token},
    runtime::{loxclass::LoxClass, loxinstance::LoxInstance},
};

use super::StringData;
use super::string_class_member::StringMembers;
#[derive(Debug, Clone, PartialEq)]
struct StringClass {
    klass: LoxClass,
    fields: Rc<RefCell<HashMap<String, InstanceField>>>,
}

impl StringClass {
    fn new(str: &StringData) -> Self {
        let members = StringMembers::new(str.clone());
        Self {
            klass: LoxClass::new(
                "String",
                members.get_methods(),
                members.get_statics(),
                members.get_fields(),
            ),
            fields: Rc::new(RefCell::new(members.get_fields())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxString {
    pub string: StringData,
    str_class: Rc<StringClass>,
    str_inst: Rc<LoxInstance>
}

impl LoxString {
    pub fn new(string: String) -> Self {
        let str = Rc::new(RefCell::new(string));
        let cl = StringClass::new(&str);
        let inst = LoxInstance::new(&cl.klass, cl.fields.clone());
        Self {
            string: str,
            str_inst: Rc::new(inst),
            str_class: Rc::new(cl),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Literal, LoxResult> {
        return self.str_inst.get(name, &self.str_inst)
    }
}
