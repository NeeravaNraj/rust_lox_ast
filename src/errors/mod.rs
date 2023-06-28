#[allow(non_snake_case)]
pub mod ParseError;
#[allow(non_snake_case)]
pub mod LoxErrorHandler;
#[allow(non_snake_case)]
pub mod RuntimeError;

use crate::lexer::token::Token;

#[allow(dead_code)]
pub enum LoxErrorsTypes {
    LexerError(String),
    SyntaxError(String),
    ParseError(String),
    RuntimeError(String),
    TypeError(String)
}

impl LoxErrorsTypes {
    fn get_error_message(err_type: &LoxErrorsTypes) -> String {
        match err_type {
            LoxErrorsTypes::LexerError(string)   => string.to_string(),
            LoxErrorsTypes::ParseError(string)   => string.to_string(),
            LoxErrorsTypes::SyntaxError(string)  => string.to_string(),
            LoxErrorsTypes::RuntimeError(string) => string.to_string(),
            LoxErrorsTypes::TypeError(string)    => string.to_string()
        }
    }

    fn confirm_error_type(err_type: &LoxErrorsTypes) -> String {
        match err_type {
            LoxErrorsTypes::LexerError(_)   => LoxErrorsTypes::LexerError("".to_string()).to_string(),
            LoxErrorsTypes::SyntaxError(_)  => LoxErrorsTypes::SyntaxError("".to_string()).to_string(),
            LoxErrorsTypes::ParseError(_)   => LoxErrorsTypes::ParseError("".to_string()).to_string(),
            LoxErrorsTypes::RuntimeError(_) => LoxErrorsTypes::RuntimeError("".to_string()).to_string(),
            LoxErrorsTypes::TypeError(_)    => LoxErrorsTypes::TypeError("".to_string()).to_string(),
        }
    }
}

impl ToString for LoxErrorsTypes {
    fn to_string(&self) -> String {
        match self {
            Self::LexerError(_)        => "LexerError".to_string(),
            Self::ParseError(_)        => "ParseError".to_string(),
            Self::SyntaxError(_)       => "SyntaxError".to_string(),
            Self::RuntimeError(_)      => "RuntimeError".to_string(),
            Self::TypeError(_)         => "TypeError".to_string(),
        }
    }
}

#[allow(dead_code)]
pub enum LoxWarningTypes{}

#[allow(dead_code)]
pub struct LoxWarning {
    pub has_warning: bool,
    pub warning_type: LoxWarningTypes,
    pub warning_message: String,
    pub line: i32
}

pub struct LoxError {
    pub has_error: bool,
    pub error_type: LoxErrorsTypes,
    pub line: i32,
    pub token: Option<Token>
}

impl LoxError {
    pub fn new(
        error_type: LoxErrorsTypes,
        token: Option<Token>,
        line: i32,
        has_error: bool
    ) -> Self {
        Self {
            error_type,
            token,
            line,
            has_error,
        }
    }
}
