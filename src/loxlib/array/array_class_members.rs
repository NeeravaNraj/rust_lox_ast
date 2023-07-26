use std::collections::HashMap;

use crate::{lexer::literal::Literal, runtime::loxinstance::InstanceField};
use super::{members::*, ArrayData};

pub struct ArrayMembers {
    array: ArrayData
}

impl ArrayMembers {
    pub fn new(array: ArrayData) -> Self {
        Self {
            array
        }
    }

    pub fn get_methods(&self) -> HashMap<String, Literal> {
        let mut map: HashMap<String, Literal> = HashMap::new();        
        map.insert(String::from("init"), init::Init::new());
        map.insert(String::from("push"), push::Push::new(self.array.clone()));
        map.insert(String::from("len"), len::Len::new(self.array.clone()));
        map.insert(String::from("pop"), pop::Pop::new(self.array.clone()));
        map.insert(String::from("replace"), replace::Replace::new(self.array.clone()));
        map.insert(String::from("insert"), insert::Insert::new(self.array.clone()));
        map.insert(String::from("delete"), delete::Delete::new(self.array.clone()));
        map
    }

    pub fn get_statics(&self) -> HashMap<String, Literal> {
        HashMap::new()
    }

    pub fn get_fields(&self) -> HashMap<String, InstanceField> {
        HashMap::new()
    }
}
