pub mod loxerrorhandler;

use crate::lexer::{literal::Literal, token::Token};

#[derive(Debug, PartialEq)]
pub enum LoxErrorsTypes {
    Lexer(String),
    Syntax(String),
    Parse(String),
    Runtime(String),
    Type(String),
    System(String),
}

impl LoxErrorsTypes {
    fn get_error_message(err_type: &LoxErrorsTypes) -> String {
        match err_type {
            LoxErrorsTypes::Lexer(string) => string.to_string(),
            LoxErrorsTypes::Parse(string) => string.to_string(),
            LoxErrorsTypes::Syntax(string) => string.to_string(),
            LoxErrorsTypes::Runtime(string) => string.to_string(),
            LoxErrorsTypes::Type(string) => string.to_string(),
            LoxErrorsTypes::System(string) => string.to_string(),
        }
    }

    fn confirm_error_type(err_type: &LoxErrorsTypes) -> String {
        match err_type {
            LoxErrorsTypes::Lexer(_) => LoxErrorsTypes::Lexer("".to_string()).to_string(),
            LoxErrorsTypes::Syntax(_) => LoxErrorsTypes::Syntax("".to_string()).to_string(),
            LoxErrorsTypes::Parse(_) => LoxErrorsTypes::Parse("".to_string()).to_string(),
            LoxErrorsTypes::Runtime(_) => LoxErrorsTypes::Runtime("".to_string()).to_string(),
            LoxErrorsTypes::Type(_) => LoxErrorsTypes::Type("".to_string()).to_string(),
            LoxErrorsTypes::System(_) => LoxErrorsTypes::System("".to_string()).to_string(),
        }
    }
}

impl ToString for LoxErrorsTypes {
    fn to_string(&self) -> String {
        match self {
            Self::Lexer(_) => "LexerError".to_string(),
            Self::Parse(_) => "ParseError".to_string(),
            Self::Syntax(_) => "SyntaxError".to_string(),
            Self::Runtime(_) => "RuntimeError".to_string(),
            Self::Type(_) => "TypeError".to_string(),
            Self::System(_) => "SystemError".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LoxWarningTypes {}

#[derive(Debug, PartialEq)]
pub struct LoxWarning {
    pub has_warning: bool,
    pub warning_type: LoxWarningTypes,
    pub warning_message: String,
    pub line: i32,
}

#[derive(Debug, PartialEq)]
pub struct LoxError {
    pub has_error: bool,
    pub error_type: LoxErrorsTypes,
    pub line: i32,
    pub token: Option<Token>,
}

impl LoxError {
    pub fn new(
        error_type: LoxErrorsTypes,
        token: Option<Token>,
        line: i32,
        has_error: bool,
    ) -> Self {
        Self {
            error_type,
            token,
            line,
            has_error,
        }
    }
    pub fn system_error(message: &str) -> LoxError {
        let err = LoxError::new(LoxErrorsTypes::System(message.to_string()), None, 0, true);
        LoxError::report(&err);
        err
    }

    pub fn report(error: &LoxError) {
        eprintln!(
            "[Lox] (line:{}) {}: {}",
            error.line,
            LoxErrorsTypes::confirm_error_type(&error.error_type),
            LoxErrorsTypes::get_error_message(&error.error_type),
        );
    }
}

#[derive(Debug, PartialEq)]
pub enum LoxResult {
    Error(LoxError),
    Warning(LoxWarning),
    Break,
    Continue,
    Return(Literal),
}
