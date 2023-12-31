use crate::loxlib::array::loxarray::LoxArray;
use crate::loxlib::number::loxnumber::LoxNumber;
use crate::loxlib::string::loxstring::LoxString;
use crate::runtime::loxfunction::LoxFunction;
use crate::runtime::{loxclass::LoxClass, loxinstance::LoxInstance};

use crate::loxlib::loxnatives::*;
use core::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(Rc<LoxNumber>),
    Str(Rc<LoxString>),
    Bool(bool),
    Func(Rc<LoxFunction>),
    Native(Rc<LoxNative>),
    Class(Rc<LoxClass>),
    Instance(Rc<LoxInstance>),
    Array(Rc<LoxArray>),
    None,
    LiteralNone,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Number(num) => write!(f, "Number {{ {} }}", num.num.borrow()),
            Self::Str(str) => write!(f, "String {{ \"{}\" }}", str.string.borrow()),
            Self::None => write!(f, "none"),
            Self::Bool(bool) => write!(f, "{bool}"),
            Self::Func(_) => write!(f, "_Function_"),
            Self::Array(_) => write!(f, "Array []"),
            Self::Class(c) => write!(f, "{c}",),
            Self::Instance(i) => write!(f, "{i}"),
            Self::Native(n) => write!(f, "{n}"),
            Self::LiteralNone => write!(f, "_LiteralNone_"), // This none is for internal use only
        }
    }
}

impl Literal {
    pub fn unwrap_number(&self) -> f64 {
        if let Literal::Number(num) = self {
            return *num.num.borrow();
        }
        panic!("Recieved {} instead of \"Literal::Number()\"", self);
    }

    pub fn as_value_string(&self) -> String {
        match self {
            Self::None => "none".to_string(),
            Self::Bool(bool) => bool.to_string(),
            Self::Str(str) => str.string.borrow().to_string(),
            Self::Number(num) => num.num.borrow().to_string(),
            _ => "none".to_string(),
        }
    }

    pub fn unwrap_str(&self) -> String {
        if let Literal::Str(str) = self {
            return str.string.borrow().to_string();
        }
        panic!("Recieved {} instead of \"Literal::Str()\"", self);
    }

    pub fn get_typename(&self) -> String {
        match self {
            Self::Number(_) => "Number".to_string(),
            Self::Str(_) => "String".to_string(),
            Self::Bool(_) => "Bool".to_string(),
            Self::Func(_) => "Function".to_string(),
            Self::Class(_) => "Class".to_string(),
            Self::Array(_) => "Array".to_string(),
            _ => self.to_string(),
        }
    }

    pub fn cmp_type(&self, rhs: &Self) -> bool {
        if self.get_typename() == rhs.get_typename() {
            return true;
        }
        false
    }

    pub fn equals(self, rhs: Self) -> bool {
        if self.get_value() == rhs.get_value() {
            return true;
        }

        false
    }

    pub fn get_value(&self) -> String {
        match self {
            Self::Number(num) => num.num.borrow().to_string(),
            Self::Str(str) => str.string.borrow().to_string(),
            Self::Bool(bool) => bool.to_string(),
            Self::None => String::from("none"),
            Self::Func(func) => func.to_string(),
            Self::Class(class) => class.to_string(),
            Self::Native(n) => n.to_string(),
            Self::Instance(i) => i.to_string(),
            Self::Array(arr) => {
                let mut str = "[".to_string();
                let len = arr.array.borrow().len();
                for (i, el) in arr.array.borrow().iter().enumerate() {
                    str.push_str(&el.get_value());
                    if len > 1 && len - 1 != i {
                        str.push_str(", ");
                    }
                }
                str.push(']');
                str
            }
            Self::LiteralNone => String::from("none"),
        }
    }

    pub fn print_value(&self) {
        match self {
            Self::Number(num) => println!("{}", num.num.borrow()),
            Self::Str(str) => println!("{}", str.string.borrow()),
            Self::Bool(bool) => println!("{bool}"),
            Self::Func(func) => println!("{func}"),
            Self::Class(class) => println!("{class}"),
            Self::Instance(i) => println!("{i}"),
            Self::Native(n) => println!("{n}"),
            Self::Array(_) => println!("{}", self.get_value()),
            Self::None => println!("{}", self),
            Self::LiteralNone => println!("{}", Literal::None),
        }
    }

    pub fn dup(&self) -> Self {
        match self {
            Self::Number(num) => Self::Number(num.to_owned()),
            Self::Str(str) => Self::Str(str.clone()),
            Self::Bool(bool) => Self::Bool(bool.to_owned()),
            Self::None => Self::None,
            Self::Func(func) => Self::Func(func.clone()),
            Self::Array(arr) => Self::Array(arr.clone()),
            Self::Class(class) => Self::Class(class.clone()),
            Self::Instance(i) => Self::Instance(i.clone()),
            Self::Native(n) => Self::Native(n.clone()),
            Self::LiteralNone => Self::LiteralNone,
        }
    }
}

impl Add<Literal> for Literal {
    type Output = Result<Literal, String>;
    fn add(self, rhs: Literal) -> Self::Output {
        if self.cmp_type(&rhs) {
            if let Literal::Str(str) = self {
                return Ok(Literal::Str(Rc::new(LoxString::new(
                    str.string
                        .borrow_mut()
                        .to_string()
                        .add(&rhs.as_value_string()),
                ))));
            } else if let Literal::Number(num) = self {
                return Ok(Literal::Number(Rc::new(LoxNumber::new(
                    num.num.borrow().add(rhs.unwrap_number()),
                ))));
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
                return Ok(Literal::Number(Rc::new(LoxNumber::new(
                    num.num.borrow().sub(rhs.unwrap_number()),
                ))));
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
                return Ok(Literal::Number(Rc::new(LoxNumber::new(
                    num.num.borrow().mul(rhs.unwrap_number()),
                ))));
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
                return Ok(Literal::Number(Rc::new(LoxNumber::new(
                    num.num.borrow().div(rhs.unwrap_number()),
                ))));
            }
        }
        Err(format!("while trying to divide {} and {}", self, rhs))
    }
}
