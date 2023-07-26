use std::collections::HashMap;

use crate::{lexer::literal::Literal, runtime::loxinstance::InstanceField};
use super::{members::*, StringData};

pub struct StringMembers {
    string: StringData
}

impl StringMembers {
    pub fn new(string: StringData) -> Self {
        Self {
            string
        }
    }

    pub fn get_methods(&self) -> HashMap<String, Literal> {
        let mut map: HashMap<String, Literal> = HashMap::new();        
        map.insert(String::from("init"), init::Init::new());
        map
    }

    pub fn get_statics(&self) -> HashMap<String, Literal> {
        HashMap::new()
    }

    pub fn get_fields(&self) -> HashMap<String, InstanceField> {
        HashMap::new()
    }
}
