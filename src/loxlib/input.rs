use crate::{
    error::*, lexer::literal::Literal, runtime::callable::LoxCallable,
    runtime::interpreter::Interpreter,
};
use std::io;
use std::process;

pub struct Input;

impl LoxCallable for Input{
    fn arity(&self) -> usize {
        1
    }

    fn call(
        &self,
        _interpreter: Option<&Interpreter>,
        args: Vec<Literal>,
    ) -> Result<Literal, LoxResult> {
        let mut input = String::new();
        let string = args.get(0).expect("Input string was null");
        if string.get_typename() != "String" {
            return Err(LoxResult::Message(format!(
                "Expected string got {} for input(string)",
                string.get_typename()
            )));
        }
        print!("{}", string.get_value());
        io::Write::flush(&mut io::stdout()).expect("Flush failed in input.");
        io::stdin().read_line(&mut input).unwrap_or_else(|error| {
            eprintln!("Error: {error}");
            process::exit(1);
        });
        Ok(Literal::Str(input.trim_end().to_string()))
    }
}
