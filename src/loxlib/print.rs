use crate::{
    error::*, lexer::literal::Literal, runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};

pub struct Print;

impl LoxCallable for Print {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: Option<&Interpreter>, args: Vec<Literal>) -> Result<Literal, LoxResult> {
        for arg in args.iter() {
            print!("{}", arg.get_value());
            print!(" ");
        }
        println!("");
        Ok(Literal::None)
    }
}
