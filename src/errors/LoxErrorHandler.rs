use crate::lexer::tokentype::TokenType;

use super::{
    LoxError,
    LoxErrorsTypes,
};

pub struct LoxErrorHandler;

impl LoxErrorHandler {
    pub fn new() -> Self {
        Self 
    }

    pub fn error(&self, line: i32, err_type: LoxErrorsTypes) -> LoxError {
        let message = LoxErrorsTypes::get_error_message(&err_type);
        let error = LoxError { 
            has_error: true, 
            error_message: message, 
            error_type: err_type, 
            line,
            token: None
        };
        self.report(&error);
        error
    }

    pub fn report(&self, error: &LoxError) {
        eprintln!("[Lox] [line \"{}\"] {} {}: {}", 
                  error.line,
                  self.get_location(error),
                  LoxErrorsTypes::confirm_error_type(&error.error_type), 
                  LoxErrorsTypes::get_error_message(&error.error_type)
        );
    }

    fn get_location(&self, error: &LoxError) -> String {
        match error.token.as_ref() {
            Some(token) if token.token_type == TokenType::EOF => "at end".to_string(),
            Some(token) => format!("at {}", token.lexeme),
            None        => String::from("")
        }
    }
}

