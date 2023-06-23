use std::{
    process,
    env,
    fs,
    io
};
mod lexer;
mod parser;
mod errors;
mod tools;
mod interpreter;

use errors::LoxErrorHandler::LoxErrorHandler;
use interpreter::interpreter::Interpreter;
use lexer::scanner::*;
use parser::rdp::Parser;
use tools::ast_print::AstPrinter;

struct Lox {
    error: LoxErrorHandler 
}


impl Lox {
    fn new() -> Lox {
        Lox {
            error: LoxErrorHandler::new()
        }
    }

    fn run(&self, file: &String) {
        let mut scanner = Scanner::new(file, &self.error);
        if let Ok(tokens) = scanner.scan_tokens(){
            let mut parser = Parser::new(tokens);

            let ast_printer = AstPrinter::new();
            if let Ok(expr) = parser.parse() {
                // ast_printer.print(&expr);
                let interpreter = Interpreter::new();
                if let Ok(value)= interpreter.evaluate(&expr) {
                    value.print_value();
                }
            }
        }
    }

    fn run_file(&self, path: &String) {
        let bytes = fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!("Error while opening file: {error}");
            process::exit(1);
        });
        self.run(&bytes);
    }

    fn run_prompt(&mut self) {
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


fn main() {
    let args: Vec<_> = env::args().collect();
    
    let mut lox = Lox::new();
    if args.len() == 1 {
        lox.run_prompt();
    } else if args.len() > 1 {
        lox.run_file(&args[1]);
    } else {
        println!("Usage: cargo run [script_path] or cargo run (runs the repl)");
        process::exit(64);
    }
}
