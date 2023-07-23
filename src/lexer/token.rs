use core::fmt;
use std::fmt::Display;

use super::literal::*;
use crate::lexer::tokentype::*;

#[derive(Debug, Clone, PartialEq)]
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

    pub fn dup(&self) -> Token {
        Token {
            token_type: self.token_type,
            lexeme: self.lexeme.to_string(),
            literal: self.literal.clone(),
            line: self.line,
        }
    }
    pub fn eof(line: i32) -> Self {
        Self {
            token_type: TokenType::EOF,
            lexeme: String::from(""),
            literal: None,
            line,
        }
    }

    pub fn this() -> Self {
        Self {
            token_type: TokenType::This,
            lexeme: String::from("this"),
            literal: None,
            line: -1,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TokenType: {:?}, Lexeme: {}, Literal: {}",
            self.token_type,
            self.lexeme,
            self.literal.as_ref().unwrap_or(&Literal::LiteralNone)
        )
    }
}
