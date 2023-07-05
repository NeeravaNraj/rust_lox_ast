use std::ops::{Add, Sub, Mul, Div};
use std::fmt::{Display, Formatter};
use core::fmt;
use crate::interpreter::callable::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    Str(String),
    Bool(bool),
    Func(Callable),
    None,
    LiteralNone,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Number(num) => write!(f, "Number {{ {num} }}"),
            Self::Str(str) => write!(f, "String {{ \"{str}\" }}"),
            Self::None => write!(f, "none"),
            Self::Bool(bool) => write!(f, "{bool}"),
            Self::Func(_) => write!(f, "_Function_"),
            Self::LiteralNone => write!(f, "_LiteralNone_"), // This none is for internal use only
        }
    }
}


impl Literal {
    pub fn unwrap_number(&self) -> f64 {
        if let Literal::Number(num) = self {
            return *num;
        }
        panic!("Recieved {} instead of \"Literal::Number()\"", self.to_string());
    }

    pub fn as_value_string(&self) -> String {
        match self {
            Self::None => "none".to_string(),
            Self::Bool(bool) => bool.to_string(),
            Self::Str(str) => str.to_string(),
            Self::Number(num) => num.to_string(),
            _ => "none".to_string()
        }
    }

    pub fn unwrap_str(&self) -> String {
        if let Literal::Str(string) = self {
            return string.to_string();
        }
        panic!("Recieved {} instead of \"Literal::Str()\"", self.to_string());
    }


    pub fn get_typename(&self) -> String {
        match self {
            Self::Number(_) => "Number".to_string(),
            Self::Str(_) => "String".to_string(),
            Self::Bool(_) => "Bool".to_string(),
            _ => self.to_string()
        }
    }

    pub fn cmp_type(&self, rhs: &Self) -> bool {
        if self.get_typename() == rhs.get_typename() {
            return true;
        }
        false
    }

    pub fn equals(self, rhs: Self) -> bool {
        if self == rhs {
            return true;
        } 

        false
    }

    pub fn print_value(&self) {
        match self {
            Self::Number(num) => println!("{num}"),
            Self::Str(str) => println!("{str}"),
            Self::Bool(bool) => println!("{bool}"),
            Self::Func(func) => println!("{}", func.to_string()),
            Self::None => println!("{}", self.to_string()),
            Self::LiteralNone => println!("{}", Literal::None.to_string()) 
        }
    }
    
    pub fn dup(&self) -> Self{
        match self {
            Self::Number(num) => Self::Number(num.to_owned()),
            Self::Str(str) => Self::Str(str.to_string()),
            Self::Bool(bool) => Self::Bool(bool.to_owned()),
            Self::None => Self::None,
            Self::Func(func) => Self::Func(func.clone()),
            Self::LiteralNone => Self::LiteralNone
        }
    }
}

impl Add<Literal> for Literal {
    type Output = Result<Literal, String>;
    fn add(self, rhs: Literal) -> Self::Output {
        if self.cmp_type(&rhs) {
            if let Literal::Str(str) = self {
                return Ok(Literal::Str(str + &rhs.as_value_string()));
            } else if let Literal::Str(str) = rhs {
                return Ok(Literal::Str(self.as_value_string() + &str));
            } else if let Literal::Number(num) = self {
                return Ok(Literal::Number(num + &rhs.unwrap_number()));
            }
        }
        
        Err(format!("while trying to add {} and {}", self, rhs))
    }
}

impl Sub<Literal> for Literal {
    type Output = Result<Literal, String>;
    fn sub(self, rhs: Literal) -> Self::Output {
        if self.cmp_type(&rhs) && self.get_typename() == "Number" {
            if let Literal::Number(num) = self {
                return Ok(Literal::Number(num - &rhs.unwrap_number()));
            }
        }
        Err(format!("while trying to subtract {} and {}", self, rhs))
    }
}

impl Mul<Literal> for Literal {
    type Output = Result<Literal, String>;
    fn mul(self, rhs: Literal) -> Self::Output {
        if self.cmp_type(&rhs) && self.get_typename() == "Number" {
            if let Literal::Number(num) = self {
                return Ok(Literal::Number(num * &rhs.unwrap_number()));
            }
        }
        Err(format!("while trying to multiply {} and {}", self, rhs))
    }
}

impl Div<Literal> for Literal {
    type Output = Result<Literal, String>;
    fn div(self, rhs: Literal) -> Self::Output {
        if self.cmp_type(&rhs) && self.get_typename() == "Number" {
            if let Literal::Number(num) = self {
                return Ok(Literal::Number(num / &rhs.unwrap_number()));
            }
        }
        Err(format!("while trying to divide {} and {}", self, rhs))
    }
}
