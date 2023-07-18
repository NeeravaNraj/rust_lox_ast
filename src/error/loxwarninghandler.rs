use crate::lexer::{token::Token, tokentype::TokenType};

use super::{LoxWarning, LoxWarningTypes, LoxResult};

#[derive(Clone)]
pub struct LoxWarningHandler;

impl LoxWarningHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn warn(&self, token: &Token, warn_type: LoxWarningTypes) -> LoxResult {
        let warn = LoxWarning::new(Some(token.dup()), warn_type, token.line, true);
        self.report(&warn);
        LoxResult::Warning(warn)
    }

    pub fn simple_warning(&self, line: i32, err_type: LoxWarningTypes) -> LoxResult {
        let warn = LoxWarning {
            has_warning: true,
            warning_type: err_type,
            token: None,
            line,
        };
        self.report(&warn);
        LoxResult::Warning(warn)
    }

    pub fn report(&self, warning: &LoxWarning) {
        println!(
            "[Lox] Warning line:{} {}: {} {}",
            warning.line,
            LoxWarningTypes::confirm_warning_type(&warning.warning_type),
            LoxWarningTypes::get_warning_message(&warning.warning_type),
            self.get_location(warning),
        );
    }

    pub fn report_asc(token: &Token, warning: &LoxWarningTypes) {
        println!(
            "[Lox] Warning line:{} {}: {}",
            token.line,
            LoxWarningTypes::confirm_warning_type(warning),
            LoxWarningTypes::get_warning_message(warning),
        );
    }

    fn get_location(&self, warning: &LoxWarning) -> String {
        match warning.token.as_ref() {
            Some(token) if token.token_type == TokenType::EOF => "at end".to_string(),
            Some(token) => format!("'{}'", token.lexeme),
            None => String::from(""),
        }
    }
}
