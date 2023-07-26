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
        map.insert(String::from("len"), len::Len::new(self.string.clone()));
        map.insert(String::from("slice"), slice::Slice::new(self.string.clone()));
        map.insert(String::from("replace"), replace::Replace::new(self.string.clone()));
        map.insert(String::from("replacen"), replacen::Replacen::new(self.string.clone()));
        map.insert(String::from("toUpper"), to_upper::ToUpper::new(self.string.clone()));
        map.insert(String::from("toLower"), to_lower::ToLower::new(self.string.clone()));
        map.insert(String::from("trim"), trim::Trim::new(self.string.clone()));
        map.insert(String::from("trim_start"), trim_start::TrimStart::new(self.string.clone()));
        map.insert(String::from("trim_end"), trim_end::TrimEnd::new(self.string.clone()));
        map.insert(String::from("split"), split::Split::new(self.string.clone()));
        map
    }

    pub fn get_statics(&self) -> HashMap<String, Literal> {
        HashMap::new()
    }

    pub fn get_fields(&self) -> HashMap<String, InstanceField> {
        HashMap::new()
    }
}
