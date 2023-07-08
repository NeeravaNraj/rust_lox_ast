mod error;
mod lexer;
mod loxlib;
mod parser;
mod runtime;
mod tools;

use error::loxerrorhandler::LoxErrorHandler;
use lexer::scanner::*;
use parser::rdp::Parser;
use runtime::interpreter::Interpreter;
use std::{fs, io, process};

pub struct Lox {
    error: LoxErrorHandler,
    interpreter: Interpreter,
    is_repl: bool,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            error: LoxErrorHandler::new(),
            interpreter: Interpreter::new(),
            is_repl: false,
        }
    }

    pub fn run(&mut self, file: &str) {
        let mut scanner = Scanner::new(file, &self.error);
        if let Ok(tokens) = scanner.scan_tokens() {
            let mut parser = Parser::new(tokens);

            if let Ok(stmts) = parser.parse() {
                if let Err(_err) = self.interpreter.interpret(stmts) {};
            }
        }
    }

    pub fn run_file(&mut self, path: &String) {
        let bytes = fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!("Error while opening file: {error}");
            process::exit(1);
        });
        self.run(&bytes);
    }

    pub fn set_is_repl(&mut self, is: bool) {
        self.interpreter.set_is_repl(is);
        self.is_repl = is;
    }

    pub fn run_prompt(&mut self) {
        let mut input = String::new();

        loop {
            print!("> ");
            io::Write::flush(&mut io::stdout()).expect("Flush failed.");
            io::stdin().read_line(&mut input).unwrap_or_else(|error| {
                eprintln!("Error: {error}");
                process::exit(1);
            });
            if input.trim() == "quit" || input.trim() == "exit" {
                break;
            }

            self.run(&input);
            input.clear();
        }
    }
}
