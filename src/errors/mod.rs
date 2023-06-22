pub mod ParseError;
pub mod LoxErrorHandler;

use crate::lexer::token::Token;

pub enum LoxErrorsTypes {
    StringNotTerminated,
    CommentNotTerminated,
    UnknownCharacter,
    LexerError(String),
    ParseError(String),
    UknownLiteral,
}

impl LoxErrorsTypes {
    fn get_error_message(err_type: &LoxErrorsTypes) -> String {
        match err_type {
            LoxErrorsTypes::StringNotTerminated  => "String has not been terminated.".to_string(),
            LoxErrorsTypes::UnknownCharacter     => "Unknown character.".to_string(),
            LoxErrorsTypes::CommentNotTerminated => "Comment has not been terminated.".to_string(),
            LoxErrorsTypes::LexerError(string)   => string.to_string(),
            LoxErrorsTypes::ParseError(string)   => string.to_string(),
            LoxErrorsTypes::UknownLiteral        => "Could not identify literal.".to_string()
        }
    }

    fn confirm_error_type(err_type: &LoxErrorsTypes) -> String {
        match err_type {
            LoxErrorsTypes::LexerError(_) => LoxErrorsTypes::LexerError("".to_string()).to_string(),
            _ => "Error".to_string()
        }
    }
}

impl ToString for LoxErrorsTypes {
    fn to_string(&self) -> String {
        match self {
            Self::CommentNotTerminated => "CommentNotTerminated".to_string(),
            Self::UnknownCharacter     => "UnknownCharacter.".to_string(),
            Self::StringNotTerminated  => "StringNotTerminated".to_string(),
            Self::LexerError(_)        => "LexerError".to_string(),
            Self::ParseError(_)        => "ParseError".to_string(),
            Self::UknownLiteral        => "UnknownLiteral".to_string(),
        }
    }
}

pub enum LoxWarningTypes {

}


pub struct LoxWarning {
    pub has_warning: bool,
    pub warning_type: LoxWarningTypes,
    pub warning_message: String,
    pub line: i32
}

pub struct LoxError {
    pub has_error: bool,
    pub error_type: LoxErrorsTypes,
    pub error_message: String,
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
        let error_message = LoxErrorsTypes::get_error_message(&error_type);
        Self {
            error_type,
            token,
            line,
            has_error,
            error_message
        }
    }
}
