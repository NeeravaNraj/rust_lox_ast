use std::collections::HashMap;

use crate::{lexer::literal::Literal, runtime::loxinstance::InstanceField};
use super::{members::*, NumberData};

pub struct NumberMembers {
    number: NumberData 
}

impl NumberMembers {
    pub fn new(number: NumberData) -> Self {
        Self {
           number 
        }
    }

    pub fn get_methods(&self) -> HashMap<String, Literal> {
        let mut map: HashMap<String, Literal> = HashMap::new();        
        map.insert(String::from("init"), init::Init::new());
        map
    }

    pub fn get_statics(&self) -> HashMap<String, Literal> {
        let mut map: HashMap<String, Literal> = HashMap::new();        
        map.insert(String::from("tryParse"), tryparse::TryParse::new());
        map
    }

    pub fn get_fields(&self) -> HashMap<String, InstanceField> {
        HashMap::new()
    }
}
