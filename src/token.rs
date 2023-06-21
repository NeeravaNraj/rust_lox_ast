use core::fmt;
use std::fmt::{Display, Formatter};

use crate::tokentype::TokenType;

pub enum Literal {
    Number(f64),
    Str(String),
    None,
    True,
    False,
    LiteralNone,
}

pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: i32) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }

    pub fn eof(line:i32) -> Self {
        Self {
            token_type: TokenType::EOF,
            lexeme: String::from(""),
            literal: None,
            line
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Number(num) => write!(f, "Number {{ {} }} ", num),
            Self::Str(str) => write!(f, "String {{ \"{}\" }} ", str),
            Self::None => write!(f, "none"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::LiteralNone => write!(f, "_LiteralNone_") // This none is for internal use only
        }
    }
}

// impl ToString for Literal {
//     fn to_string(&self) -> String {
//         match self {
//             Self::Number(num) => format!("Number {{ {} }} ", num),
//             Self::Str(str) => format!("String {{ \"{}\" }} ", str),
//             Self::None => format!("none"),
//             Self::True => format!("true"),
//             Self::False => format!("false"),
//             Self::LiteralNone => format!("_LiteralNone_") // This none is for internal use only
//         }
//     }
// }

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "TokenType: {:?}, Lexeme: {}, Literal: {}", 
            self.token_type, self.lexeme, self.literal.as_ref().unwrap_or_else(|| &Literal::LiteralNone)
        )
    }
}
