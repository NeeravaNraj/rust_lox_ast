use std::collections::HashMap;

use crate::{lexer::literal::Literal, runtime::loxinstance::InstanceField};
use super::members::*;

pub struct NumberMembers;

impl NumberMembers {
    pub fn new() -> Self {
        Self {}
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
