use std::time::SystemTime;

use crate::{
    interpreter::interpreter::Interpreter,
    lexer::literal::Literal,
    error::*,
    interpreter::callable::LoxCallable
};
pub struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &Interpreter, _args: Vec<Literal>) -> Result<Literal, LoxResult> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
        match now {
            Ok(time) => Ok(Literal::Number(time.as_secs_f64())),
            Err(err) => Err(
                LoxResult::LoxError(
                    LoxError::system_error(format!("Clock return invalid duration: {}", err.duration().as_secs_f64()).as_str())
                )
            )
        }
    }

    fn to_string(&self) -> String {
        String::from("<Fn clock>")
    }
}
