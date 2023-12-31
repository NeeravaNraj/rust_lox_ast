use std::{time::SystemTime, rc::Rc};

use crate::{
    error::*, lexer::literal::Literal, runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

use super::number::loxnumber::LoxNumber;

pub struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: Option<&Interpreter>, _args: Vec<Literal>) -> Result<Literal, LoxResult> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => Ok(Literal::Number(Rc::new(LoxNumber::new(time.as_secs_f64())))),
            Err(err) => Err(LoxResult::Error(LoxError::system_error(
                format!(
                    "Clock return invalid duration: {}",
                    err.duration().as_secs_f64()
                )
                .as_str(),
            ))),
        }
    }
}
