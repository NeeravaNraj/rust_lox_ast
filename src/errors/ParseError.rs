use crate::lexer::token::Token;

use super::{
    LoxError, 
    LoxErrorHandler::LoxErrorHandler, LoxErrorsTypes
};


pub struct ParseErrorHandler {
    handler: LoxErrorHandler,
}

impl ParseErrorHandler {
    pub fn new() -> Self {
        Self {
            handler: LoxErrorHandler::new()
        }
    }

    pub fn error(&self, token: &Token, err_type: LoxErrorsTypes) -> LoxError {
        let error = LoxError::new(err_type, Some(token.dup()), token.line, true);
        self.handler.report(&error);
        error
    }
}
