use crate::lexer::{token::Token, tokentype::TokenType};

use super::{LoxError, LoxErrorsTypes, LoxResult};

#[derive(Clone)]
pub struct LoxErrorHandler;

impl LoxErrorHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn error(&self, token: &Token, err_type: LoxErrorsTypes) -> LoxResult {
        let error = LoxError::new(err_type, Some(token.dup()), token.line, true);
        self.report(&error);
        LoxResult::Error(error)
    }

    pub fn simple_error(&self, line: i32, err_type: LoxErrorsTypes) -> LoxResult {
        let error = LoxError {
            has_error: true,
            error_type: err_type,
            line,
            token: None,
        };
        self.report(&error);
        LoxResult::Error(error)
    }

    pub fn report(&self, error: &LoxError) {
        eprintln!(
            "[Lox] line:{} {}: {} {}",
            error.line,
            LoxErrorsTypes::confirm_error_type(&error.error_type),
            LoxErrorsTypes::get_error_message(&error.error_type),
            self.get_location(error),
        );
    }

    fn get_location(&self, error: &LoxError) -> String {
        match error.token.as_ref() {
            Some(token) if token.token_type == TokenType::EOF => "at end".to_string(),
            Some(token) => format!("'{}'", token.lexeme),
            None => String::from(""),
        }
    }
}
