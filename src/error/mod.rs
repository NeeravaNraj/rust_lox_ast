pub mod loxerrorhandler;

use crate::lexer::{token::Token, literal::Literal};

#[allow(dead_code)]
pub enum LoxErrorsTypes {
    LexerError(String),
    SyntaxError(String),
    ParseError(String),
    RuntimeError(String),
    TypeError(String),
    SystemError(String),
}

impl LoxErrorsTypes {
    fn get_error_message(err_type: &LoxErrorsTypes) -> String {
        match err_type {
            LoxErrorsTypes::LexerError(string)   => string.to_string(),
            LoxErrorsTypes::ParseError(string)   => string.to_string(),
            LoxErrorsTypes::SyntaxError(string)  => string.to_string(),
            LoxErrorsTypes::RuntimeError(string) => string.to_string(),
            LoxErrorsTypes::TypeError(string)    => string.to_string(),
            LoxErrorsTypes::SystemError(string)  => string.to_string(),
        }
    }

    fn confirm_error_type(err_type: &LoxErrorsTypes) -> String {
        match err_type {
            LoxErrorsTypes::LexerError(_)   => LoxErrorsTypes::LexerError("".to_string()).to_string(),
            LoxErrorsTypes::SyntaxError(_)  => LoxErrorsTypes::SyntaxError("".to_string()).to_string(),
            LoxErrorsTypes::ParseError(_)   => LoxErrorsTypes::ParseError("".to_string()).to_string(),
            LoxErrorsTypes::RuntimeError(_) => LoxErrorsTypes::RuntimeError("".to_string()).to_string(),
            LoxErrorsTypes::TypeError(_)    => LoxErrorsTypes::TypeError("".to_string()).to_string(),
            LoxErrorsTypes::SystemError(_)  => LoxErrorsTypes::SystemError("".to_string()).to_string()
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
            Self::SystemError(_)       => "SystemError".to_string()
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
    pub fn system_error(message: &str) -> LoxError{
        let err = LoxError::new(LoxErrorsTypes::SystemError(message.to_string()), None, 0, true);
        LoxError::report(&err);
        err
    }

    pub fn report(error: &LoxError) {
        eprintln!("[Lox] (line:{}) {}: {}", 
              error.line,
              LoxErrorsTypes::confirm_error_type(&error.error_type), 
              LoxErrorsTypes::get_error_message(&error.error_type),
        );
    }
}

pub enum LoxResult {
    LoxError(LoxError),
    LoxWarning(LoxWarning),
    LoxBreak,
    LoxContinue,
    LoxReturn(Literal),
}
