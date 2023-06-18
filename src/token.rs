use core::fmt;
use std::fmt::{Display, Formatter};

use crate::tokentype::TokenType;

pub enum Literal {
    Number(f64),
    Str(String),
    Nil,
    True,
    False,
    LiteralNone,
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: i32,
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
            Self::Number(num) => write!(f, "Number {}", num),
            Self::Str(str) => write!(f, "String \"{}\"", str),
            Self::Nil => write!(f, "Nil"),
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
            Self::LiteralNone => write!(f, "_None_") // This none is for internal use only
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "TokenType: {:?}, Lexeme: \"{}\", Literal: {}", 
            self.token_type, self.lexeme, self.literal.as_ref().unwrap_or_else(|| &Literal::LiteralNone)
        )
    }
}
